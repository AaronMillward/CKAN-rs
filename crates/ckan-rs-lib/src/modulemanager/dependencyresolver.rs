//! module for creating a list of top level requirements and generating a list of required ckan modules

use std::collections::{HashMap, HashSet, VecDeque};

use crate::metadb::ckan::*;
use crate::metadb::DescriptorMatchesExt;
use crate::metadb::KspVersionMatchesExt;
use crate::metadb::MetaDB;


#[derive(Debug, Default)]
pub struct InstallRequirement {
	pub mod_identifier: String,
	pub required_version: Option<ModVersion>
}

mod mutliple_providers_decision;
pub use mutliple_providers_decision::MutlipleProvidersDecision;
pub use mutliple_providers_decision::MutlipleProvidersDecisionFinal;
pub use mutliple_providers_decision::MutlipleProvidersDecisionValidation;

#[derive(Debug)]
pub enum RelationshipProcess {
	/// There are more steps to be done.
	Incomplete,
	/// A module was virtual and requires a decision to progress.
	MultipleProviders(MutlipleProvidersDecision),
	/// There are more steps required but there are unresolved issues preventing further steps.
	Halt,
	/// The resolver is done
	Complete,
}

/// Describes why a given module or identifier failed to resolve.
#[derive(Debug)]
pub enum FailedResolve<'db> {
	/// All possible candidates for the identifier are not compatible with the current requirements.
	NoCompatibleCandidates(String),
	/// These two modules cannot be installed together.
	ModulesConflict(&'db Ckan, &'db Ckan),
	/// The identifier does exist but no version supports the compatible game versions.
	NoCompatibleKspVersion(String),
	/// There are no compatible modules matching this identifier.
	NoCompatibleModules(String),
	/// The given identifier is not present in the database.
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

	compatible_ksp_versions: Vec<KspVersion>,
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
			compatible_ksp_versions,
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
							MutlipleProvidersDecision::new(
								any_of.into_iter().map(|r| r.name).collect()
							)
						)
					}
				},
				Relationship::One(r) => {
					r
				},
			}
		};

		dbg!(&current_descriptor);

		/* This maybe be empty if the identifier does not exist at all */
		let mut compatible_modules = { 
			let mut desc_match = self.metadb.get_modules().iter()
				.descriptor_matches(current_descriptor.clone())
				.peekable();

			if desc_match.peek().is_none() {
				self.failed.push(FailedResolve::IdentifierDoesNotExist(current_descriptor.name));
				self.resolve_queue.remove(0);
				return RelationshipProcess::Incomplete
			}

			/* Modules appear to not put version restrictions on their virtual identifiers, so we have to filter to only compatible game versions.
			 * For example, Parallax 2.0.0 requires any version of Parallax-Textures
			 * which is provided by BeyondHome however BeyondHome isn't compatible with
			 * Parallax 2.0.0's game version
			 *
			 * This does mean that all dependencies of a module have to meet the game version requirements.
			 */
			/* TODO: We could add some kind of decision for if game version should be checked */
			desc_match
				.ksp_version_matches(self.compatible_ksp_versions.clone())
				.collect::<Vec<_>>()
		};

		if compatible_modules.is_empty() {
			self.failed.push(FailedResolve::NoCompatibleKspVersion(current_descriptor.name));
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
					self.resolved_virtual_identifiers.insert(current_descriptor.name, module);
					self.resolve_queue.remove(0);
					return RelationshipProcess::Incomplete
				}
				
				else if let Some(m_id) = self.decisions.iter().find(|s| providers.contains(*s)) {
					compatible_modules = compatible_modules.into_iter().filter(|m| &m.identifier == m_id).collect();
				}
				
				else {
					return RelationshipProcess::MultipleProviders(
						MutlipleProvidersDecision::new(providers)
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
			self.failed.push(FailedResolve::NoCompatibleCandidates(current_descriptor.name));
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

	pub fn add_decision(&mut self, decision: MutlipleProvidersDecisionFinal) {
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