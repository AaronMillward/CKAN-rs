//! module for creating a list of top level requirements and generating a list of required ckan modules

use std::collections::{HashMap, HashSet, VecDeque};

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
	IdentifierDoesNotExist(String),
}

/// RelationshipResolver will take a list of top level requirements and generate a list of required modules
#[derive(Debug)]
pub struct RelationshipResolver<'db> {
	metadb: &'db MetaDB,
	/// Tells the resolver which module to chose when faced with a decision
	decisions: HashSet<String>,
	/// Identifiers to be resolved
	/* XXX: Maybe use `Cow` here instead? it would save a lot of string cloning 
	 * It currrently takes ownership because the InstallRequirements are converted to Relationships making referencing awkward
	 */
	resolve_queue: VecDeque<Relationship>,
	/// Identifiers that have been resolved
	resolved_virtual_identifiers: HashMap<String, &'db Ckan>,
	/// Final list of modules that satisfy the requirements
	confirmed: HashSet<&'db Ckan>,
	/// List of failed resolves, must be empty for `confirmed` to be valid
	failed: Vec<FailedResolve<'db>>,
}

impl<'db> RelationshipResolver<'db> {
	pub fn new(
		mut compatible_ksp_versions: Vec<KspVersion>,
		requirements: Vec<InstallRequirement>,
		metadb: &'db MetaDB
	) -> Self {
		/* We intialize these with the requirements as they certainly have to exist */
		let mut queue = VecDeque::<Relationship>::new();
		
		for req in &requirements {
			queue.push_back(
				Relationship::One(ModuleDescriptor {
					name: req.mod_identifier.clone(),
					version: req.required_version.clone(),
					min_version: None,
					max_version: None,
				})
			);
		}

		/* TODO: Don't panic */
		if compatible_ksp_versions.is_empty() {
			panic!("compatible_ksp_versions can't be empty") 
		}
		compatible_ksp_versions.sort();
		compatible_ksp_versions.dedup();

		Self {
			metadb,
			decisions: Default::default(),
			resolve_queue: queue,
			resolved_virtual_identifiers: Default::default(),
			confirmed: Default::default(),
			failed: Default::default(),
		}
	}

	/// Moves the resolver forward allowing caller to handle decisions and errors.
	/// 
	/// Uses a breadth first approach so higher level decisions are made first.
	pub fn step(&mut self) -> RelationshipProcess {

		fn is_relationship_already_fulfilled(resolver: &RelationshipResolver, relationship: &Relationship) -> bool {
			for desc in relationship.as_vec() {
				if let Some(m) = resolver.resolved_virtual_identifiers.get(&desc.name) {
					if does_module_fulfill_relationship(m, relationship) {
						return true
					} else {
						/* XXX: is else an error here? the installed module doesn't fit the bounds */
						/* So it should be uninstalled added back into the queue? */
						todo!()
					}
				}
			}

			if resolver.confirmed.iter().any(|m| does_module_fulfill_relationship(m, relationship)) {
				return true
			}

			false
		}

		/* Get next descriptor */
		let current_descriptor = {
			let processing_relationship = {
				let opt = self.resolve_queue.get(0).cloned();
				if opt.is_none() {
					if self.failed.is_empty() {
						return RelationshipProcess::Complete
					} else {
						/* We've reached the end of the resolve queue but there are errors with the resolve so we can't say it's completed */
						return RelationshipProcess::Halt
					}
				}
				opt.unwrap()
			};
			
			if is_relationship_already_fulfilled(self, &processing_relationship) {
				self.resolve_queue.remove(0);
				return RelationshipProcess::Incomplete
			}
			
			/* Now we determine which descriptor to install */
			/* We don't need to handle virtual identifiers here, that is handled later when handling the descriptor */
			match processing_relationship {
				Relationship::AnyOf(any_of) => { /* Ask the caller for a decision if there are multiple choices */
					let mut entry: Option<ModuleDescriptor> = None;
					for desc in &any_of {
						if self.decisions.contains(&desc.name) {
							entry = Some(desc.clone());
							break;
						}
					}
	
					if let Some(e) = entry {
						e
					} else {
						return RelationshipProcess::MultipleProviders(
							RelationshipMutlipleProvidersDecision {
								options: any_of.into_iter().map(|r| r.name).collect(),
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

		dbg!(&current_descriptor);

		/* XXX: Here we assume the descriptors never produce game version incompatibility */
		/* TODO: *sigh* they do... */
		/* This maybe be empty if the identifier does not exist at all */
		let mut compatible_modules = self.metadb.get_modules_matching_descriptor(&current_descriptor);

		if compatible_modules.is_empty() {
			self.failed.push(FailedResolve::IdentifierDoesNotExist(current_descriptor.name.clone()));
			self.resolve_queue.remove(0);
			return RelationshipProcess::Incomplete
		}

		/* Handle virtual identifiers which will produce multiple identifiers */
		{
			let id = &compatible_modules.get(0).unwrap().identifier;
			if compatible_modules.iter().any(|m| &m.identifier != id) {
				let providers = compatible_modules.iter().map(|module| module.identifier.clone()).collect::<HashSet<_>>();

				debug_assert!( providers.len() > 1 );

				/* Check confirmed to see if decision has already been made */
				if let Some(module) = self.confirmed.iter().find(|c| providers.contains(&c.identifier)) {
					self.resolved_virtual_identifiers.insert(current_descriptor.name.clone(), module);
					self.resolve_queue.remove(0);
					return RelationshipProcess::Incomplete
				}
				
				else if let Some(m_id) = self.decisions.iter().find(|s| providers.contains(*s)) {
					compatible_modules = compatible_modules.into_iter().filter(|m| &m.identifier == m_id).collect();
				}
				
				else {
					return RelationshipProcess::MultipleProviders(
						RelationshipMutlipleProvidersDecision {
							options: providers,
							selection: "".to_string(),
						}
					)
				}
			}
		}

		/* We sort the modules so the latest versions are at the start of the vec */
		compatible_modules.sort();
		compatible_modules.reverse();

		let mut any_conflict = false;
		for candidate in compatible_modules {
			let mut conflicts = false;
			for module in &self.confirmed {
				if Ckan::do_modules_conflict(candidate, module) {
					any_conflict = true;
					self.failed.push(FailedResolve::ModulesConflict(candidate, module));
					conflicts = true;
				}
			}

			if conflicts {
				self.resolve_queue.remove(0);
			} else {
				Self::confirm_module(&mut self.resolve_queue, &mut self.confirmed, candidate);
				self.resolve_queue.remove(0);
				return RelationshipProcess::Incomplete
			}
		}

		if !any_conflict {
			self.failed.push(FailedResolve::NoCompatibleCandidates(current_descriptor.name.clone()));
		}

		RelationshipProcess::Incomplete
	}

	fn confirm_module(resolve_queue: &mut VecDeque<Relationship>, confirmed: &mut HashSet<&'db Ckan>, module: &'db Ckan) {
		for dep in &module.depends {
			eprintln!("module {} adding dep {:?}", module.identifier, dep);
			resolve_queue.push_back(dep.clone())
		}
		confirmed.insert(module);
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