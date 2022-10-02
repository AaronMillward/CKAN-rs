//!

use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

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
	virtual_identifier: String,
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
	/// Maps virtual identifiers to a real module
	decisions: HashMap<String, String>,
	/// Identifiers to be resolved
	resolve_queue: VecDeque<String>,
	/// Identifiers that have been resolved
	completed: HashSet<String>,
	/// Final list of modules that satisfy the requirements
	confirmed: HashSet<&'db Ckan>,
	/// List of failed resolves, must be empty for `confirmed` to be valid
	failed: Vec<FailedResolve<'db>>,
}

impl<'db> RelationshipResolver<'db> {
	pub fn new(
		compatible_ksp_versions: HashSet<KspVersion>,
		requirements: Vec<InstallRequirement>,
		metadb: &'db MetaDB
	) -> Self {
		/* We intialize these with the requirements as they certainly have to exist */
		let mut confirmed = HashSet::<&Ckan>::new();
		let mut queue = VecDeque::<String>::new();
		let mut failed = Vec::<FailedResolve>::new();
		
		for req in &requirements {
			/* if we have a specific version we can get the module immediately otherwise we add it to the queue to be resolved */
			if let Some(ver) = &req.required_version {
				if let Some(c) = metadb.get_from_identifier_and_version(&req.mod_identifier, ver) {
					confirmed.insert(c);
				} else {
					failed.push(FailedResolve::NoCompatibleCandidates(req.mod_identifier.clone()))
				}
			} else {
				queue.push_front(req.mod_identifier.clone());
			}
		}

		Self {
			metadb,
			compatible_ksp_versions,
			decisions: Default::default(),
			resolve_queue: queue,
			completed: Default::default(),
			confirmed,
			failed,
		}
	}

	/// Moves the resolver forward allowing caller to handle decisions and errors.
	/// 
	/// Uses a breadth first approach so higher level decisions are made first.
	pub fn step(&mut self) -> RelationshipProcess {
		let processing_identifier = {
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

		if self.completed.contains(processing_identifier) {
			return RelationshipProcess::Incomplete
		}

		let modules_ksp_compatible = self.metadb.get_modules().iter().filter(|module| {
			/* TODO: ksp_version_strict | this needs to be fixed in KspVersion */
			if let Some(version) = &module.ksp_version {
				return self.compatible_ksp_versions.contains(version);
			}
			match (&module.ksp_version_min, &module.ksp_version_max) {
				(None, None) => {
					/* XXX: at this point we can deduce the module has no ksp version requirements so we assume it doesn't care and is compatible with all versions. */
					true
				},
				(None, Some(max)) => {
					let mut one_version_meets_requirement = false;
					for ksp in &self.compatible_ksp_versions {
						one_version_meets_requirement &= ksp < max;
					}
					one_version_meets_requirement
				},
				(Some(min), None) => {
					let mut one_version_meets_requirement = false;
					for ksp in &self.compatible_ksp_versions {
						one_version_meets_requirement &= ksp > min;
					}
					one_version_meets_requirement
				},
				(Some(min), Some(max)) => {
					let mut one_version_meets_requirement = false;
					for ksp in &self.compatible_ksp_versions {
						one_version_meets_requirement &= min < ksp && ksp < max;
					}
					one_version_meets_requirement
				},
			}
		}).collect::<Vec<_>>();

		/* `compatible_modules` maybe be empty for two reasons:
		 * 1. The identifier is virtual so no modules actually implement it
		 * 2. The identifier has no versions compatible with the compatible ksp versions
		 *
		 * Because of this the only way to detect case 2 is by checking if case 1 is true
		 */
		let mut compatible_modules = modules_ksp_compatible.iter().filter(|module| &module.identifier == processing_identifier).collect::<Vec<_>>();

		/* Handle virtual modules */
		{
			let is_virtual = compatible_modules.is_empty();
	
			if is_virtual {
				if !self.decisions.contains_key(processing_identifier) {
					let providers = self.metadb.get_modules().iter()
						.filter(|module| module.provides.contains(processing_identifier))
						.map(|module| module.identifier.clone())
						.collect::<HashSet<_>>();

					/* Handle no providers case. See comment above `compatible_modules` for more info */
					if providers.is_empty() {
						self.failed.push(FailedResolve::NoCompatibleKspVersion(processing_identifier.clone()));
						return RelationshipProcess::Incomplete
					}

					if providers.len() == 1 {
						let new_id = providers.iter().collect::<Vec<_>>()[0];
						compatible_modules = modules_ksp_compatible.iter().filter(|module| &module.identifier == new_id).collect::<Vec<_>>();
					} else {
						/* Check confirmed to see if decision has already been made */
						if self.confirmed.iter().any(|c| providers.contains(&c.identifier)) {
							self.resolve_queue.remove(0);
							return RelationshipProcess::Incomplete
						} else {
							return RelationshipProcess::MultipleProviders(
								RelationshipMutlipleProvidersDecision {
									virtual_identifier: processing_identifier.clone(),
									options: providers,
									selection: "".to_string(),
								}
							)
						}
					}
				} else {
					let new_identifier = self.decisions.get(processing_identifier).unwrap().clone();
					compatible_modules = modules_ksp_compatible.iter().filter(|module| module.identifier == new_identifier).collect::<Vec<_>>();
				}
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
				self.confirmed.insert(candidate);
			}
			self.resolve_queue.remove(0);
		} else {
			/* There are no compatible modules  */
			self.failed.push(FailedResolve::NoCompatibleCandidates(processing_identifier.clone()));
			return RelationshipProcess::Incomplete
		}

		RelationshipProcess::Incomplete
	}

	pub fn add_decision(&mut self, decision: RelationshipMutlipleProvidersDecision) {
		self.decisions.insert(decision.virtual_identifier, decision.selection);
	}

	pub fn get_failed_resolves(&self) -> &Vec<FailedResolve> {
		&self.failed
	}
}