//!

use std::collections::{HashSet, VecDeque};

use crate::metadb::ckan::*;
use crate::metadb::MetaDB;

#[derive(Debug, Default)]
pub struct InstallRequirement {
	pub mod_identifier: String,
	pub required_version: Option<ModVersion>
}

/// Describes a decision to be made by the user when mutiple providers are available for a given module
#[derive(Debug)]
pub struct RelationshipMutlipleProvidersDecision {
	/* TODO: reason field */
	options: HashSet<String>,
	selection: String,
}

impl RelationshipMutlipleProvidersDecision {
	pub fn get_options(&self) -> &HashSet<String> {
		&self.options
	}

	pub fn select(&mut self, choice: String) -> bool {
		if !self.options.contains(&choice) {
			false
		} else {
			self.selection = choice;
			true
		}
	}

	/* TODO: Some `finalize` function to stop users from passing incomplete decisions back to the resolver */
}

#[derive(Debug)]
pub enum RelationshipProcess {
	/// There are more steps to be done.
	Incomplete,
	/// A module was virtual and requires a decision to progress.
	MultipleProviders(RelationshipMutlipleProvidersDecision),
	/// There are more steps required but there are unresolved issues preventing further steps.
	Halt,
	/// The resolver is done
	Complete,
}

/// Describes why a given module or identifier failed to resolve.
#[derive(Debug)]
pub enum FailedResolve<'db> { 
	NoCompatibleCandidates(String),
	ModulesConflict(&'db Ckan, &'db Ckan),
	NoCompatibleKspVersion(String),
}

/// DependencyResolver will take a list of top level requirements and generate a list of required modules
#[derive(Debug)]
pub struct RelationshipResolver<'db> {
	metadb: &'db MetaDB,
	compatible_ksp_versions: HashSet<KspVersion>,
	/// Tells the resolver which module to chose when faced with a decision
	decisions: HashSet<String>,
	/// Identifiers to be resolved
	/* XXX: Maybe use `Cow` here instead? it would save a lot of string cloning 
	 * It currrently takes ownership because the InstallRequirements are converted to Relationships making referencing awkward
	 */
	resolve_queue: VecDeque<Relationship>,
	/// Identifiers that have been resolved
	completed: HashSet<String>,
	/// Final list of modules that satisfy the requirements
	confirmed: HashSet<&'db Ckan>,
	/// List of failed resolves, must be empty for `confirmed` to be valid
	failed: Vec<FailedResolve<'db>>,
	/// List of modules narrowed to compatible ksp versions
	modules_ksp_compatible: Vec<&'db Ckan>,
}

impl<'db> RelationshipResolver<'db> {
	pub fn new(
		compatible_ksp_versions: HashSet<KspVersion>,
		requirements: Vec<InstallRequirement>,
		metadb: &'db MetaDB
	) -> Self {
		/* We intialize these with the requirements as they certainly have to exist */
		let mut queue = VecDeque::<Relationship>::new();
		
		for req in &requirements {
			queue.push_back(
				Relationship::One(RelationshipEntry {
					name: req.mod_identifier.clone(),
					version: req.required_version.clone(),
					min_version: None,
					max_version: None,
				})
			);
		}

		let modules_ksp_compatible = metadb.get_modules().iter().filter(|module| {
			/* TODO: ksp_version_strict | this needs to be fixed in KspVersion */
			if let Some(version) = &module.ksp_version {
				return compatible_ksp_versions.contains(version);
			}
			match (&module.ksp_version_min, &module.ksp_version_max) {
				(None, None) => {
					/* XXX: at this point we can deduce the module has no ksp version requirements so we assume it doesn't care and is compatible with all versions. */
					true
				},
				(None, Some(max))      => { compatible_ksp_versions.iter().any(|ksp| ksp < max) },
				(Some(min), None)      => { compatible_ksp_versions.iter().any(|ksp| ksp > min) },
				(Some(min), Some(max)) => { compatible_ksp_versions.iter().any(|ksp| min < ksp && ksp < max) },
			}
		}).collect::<Vec<_>>();

		Self {
			metadb,
			compatible_ksp_versions,
			decisions: Default::default(),
			resolve_queue: queue,
			completed: Default::default(),
			confirmed: Default::default(),
			failed: Default::default(),
			modules_ksp_compatible,
		}
	}

	/// Moves the resolver forward allowing caller to handle decisions and errors.
	/// 
	/// Uses a breadth first approach so higher level decisions are made first.
	pub fn step(&mut self) -> RelationshipProcess {
		/* Handle next relationship */
		let current_relationship_entry = {
			let processing_relationship = {
				let opt = self.resolve_queue.get(0);
				if opt.is_none() {
					if self.failed.is_empty() {
						return RelationshipProcess::Complete
					} else {
						return RelationshipProcess::Halt
					}
				}
				opt.unwrap()
			};
	
			if processing_relationship.as_vec().iter().any(|r| self.completed.contains(&r.name)) {
				/* TODO: check that the completed module fulfills the relationship requirement */
				self.resolve_queue.remove(0);
				return RelationshipProcess::Incomplete
			}
	
			match processing_relationship {
				Relationship::AnyOf(v) => {
					let mut entry: Option<&RelationshipEntry> = None;
					for r in v {
						if self.decisions.contains(&r.name) || self.completed.contains(&r.name) {
							entry = Some(r);
							break;
						}
					}
	
					if let Some(e) = entry {
						e
					} else {
						return RelationshipProcess::MultipleProviders(
							RelationshipMutlipleProvidersDecision {
								options: v.iter().map(|r| r.name.clone()).collect(),
								selection: "".to_string(),
							}
						)
					}
				},
				Relationship::One(r) => {
					r
				},
			}
		};

		

		/* `compatible_modules` maybe be empty for multiple reasons:
		 * 1. The identifier is virtual so no modules actually implement it
		 * 2. The identifier has no versions compatible with the compatible ksp versions
		 * 3. The identifier does not exist at all
		 *
		 * Because of this the only way to detect case 2 is by checking if case 1 is true
		 */
		let mut compatible_modules = self.modules_ksp_compatible.iter().filter(|module| module.identifier == current_relationship_entry.name).collect::<Vec<_>>();

		/* Handle virtual modules */
		if compatible_modules.is_empty() {
			if !self.decisions.contains(&current_relationship_entry.name) {
				/* FIXME: by using `modules_ksp_compatible` we make it impossible to tell the difference between case 2 and 3. compare with all modules to make this clear */
				let providers = self.modules_ksp_compatible.iter()
					.filter(|module| module.provides.contains(&current_relationship_entry.name))
					.map(|module| module.identifier.clone())
					.collect::<HashSet<_>>();

				/* Handle no providers case. See comment above `compatible_modules` for more info */
				if providers.is_empty() {
					self.failed.push(FailedResolve::NoCompatibleKspVersion(current_relationship_entry.name.clone()));
					return RelationshipProcess::Incomplete
				}

				if providers.len() == 1 {
					let new_id = providers.iter().collect::<Vec<_>>()[0];
					compatible_modules = self.modules_ksp_compatible.iter().filter(|module| &module.identifier == new_id).collect::<Vec<_>>();
				} else {
					/* Check confirmed to see if decision has already been made */
					if self.confirmed.iter().any(|c| providers.contains(&c.identifier)) {
						self.resolve_queue.remove(0);
						return RelationshipProcess::Incomplete
					} else {
						return RelationshipProcess::MultipleProviders(
							RelationshipMutlipleProvidersDecision {
								options: providers,
								selection: "".to_string(),
							}
						)
					}
				}
			} else {
				let new_identifier = self.decisions.get(&current_relationship_entry.name).unwrap();
				compatible_modules = self.modules_ksp_compatible.iter().filter(|module| &module.identifier == new_identifier).collect::<Vec<_>>();
			}
		}

		/* We sort the modules so the latest versions are at the start of the vec */
		compatible_modules.sort_by(|lhs, rhs| rhs.partial_cmp(lhs).unwrap_or(std::cmp::Ordering::Equal));

		/* TODO: Iterate all compatible modules instead of just the latest */
		if let Some(candidate) = compatible_modules.get(0) {
			let mut conflicts = false;
			for module in &self.confirmed {
				if Ckan::do_modules_conflict(candidate, module) {
					self.failed.push(FailedResolve::ModulesConflict(candidate, module));
					conflicts = true;
				}
			}

			if !conflicts {
				#[allow(mutable_borrow_reservation_conflict)] /* It's okay here as long as confirm_module does not alter modules_ksp_compatible */
				/* FIXME: Find a way of doing this without the warning, maybe confirm_module takes the fields as arguments instead? */
				self.confirm_module(candidate);
			}
			self.resolve_queue.remove(0);
		} else {
			/* There are no compatible modules  */
			self.failed.push(FailedResolve::NoCompatibleCandidates(current_relationship_entry.name.clone()));
			return RelationshipProcess::Incomplete
		}

		RelationshipProcess::Incomplete
	}

	fn confirm_module(&mut self, module: &'db Ckan) {
		for dep in &module.depends {
			self.resolve_queue.push_back(dep.clone())
		}
		self.completed.insert(module.identifier.clone());
		self.confirmed.insert(module);
	}

	pub fn add_decision(&mut self, decision: RelationshipMutlipleProvidersDecision) {
		self.decisions.insert(decision.selection);
	}

	pub fn get_failed_resolves(&self) -> &Vec<FailedResolve> {
		&self.failed
	}

	fn is_complete(&self) -> bool {
		self.resolve_queue.is_empty() && self.failed.is_empty()
	}

	/// When the resolver is completed this method will return `Some` containing all calculated modules
	pub fn get_final_module_list(&self) -> Option<&HashSet<&Ckan>> {
		if self.is_complete() {
			Some(&self.confirmed)
		} else {
			None
		}
	}
}