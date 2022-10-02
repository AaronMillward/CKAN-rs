//!

use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

use crate::metadb::ckan::*;
use crate::metadb::MetaDB;

#[derive(Debug, Default)]
pub struct InstallRequirement {
	mod_identifier: String,
	required_version: Option<ModVersion>
}

/// Describes a decision to be made by the user when mutiple providers are available for a given module
#[derive(Debug)]
pub struct RelationshipMutlipleProvidersDecision<'a> {
	inner: RelationshipResolver<'a>,
	virtual_identifier: String,
	options: HashSet<String>,
	selection: String,
}

impl<'a> RelationshipMutlipleProvidersDecision<'a> {
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

	pub fn confirm(mut self) -> RelationshipResolver<'a> {
		self.inner.add_decision(self.virtual_identifier, self.selection);
		self.inner
	}
}

#[derive(Debug)]
pub enum RelationshipProcess<'a> {
	/// There are more steps to be done.
	Incomplete(RelationshipResolver<'a>),
	/// A module was virtual and requires a decision to progress.
	MultipleProviders(RelationshipMutlipleProvidersDecision<'a>),
	/// There are more steps required but there are unresolved issues preventing further steps.
	Halt(RelationshipResolver<'a>),
	/// The resolver is done
	Complete(RelationshipResolver<'a>),
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

	/// moves the resolver forward allowing caller to handle conflicts and decisions
	pub fn step(mut self) -> RelationshipProcess<'db> {
		/* 
		- Select depends, conflicts, ksp_version for each mod identifier
		- if requirement has a specific version limit select to that version
		- if there are no matching versions we have an error
		- recurse until no new deps are found
		*/

		/* We use a breadth first approach as I think users will prefer to make higher level decisions first */

		/* Pop next in queue */
		/* If it is a virtual identifier ask which option */
		/* add deps of the module to the queue if they aren't in completed already */
		/* repeat step 1 until no queue is empty */

		let identifier = {
			let opt = self.resolve_queue.get(0);
			if opt.is_none() {
				if self.failed.is_empty() {
					return RelationshipProcess::Complete(self)
				} else {
					return RelationshipProcess::Halt(self)
				}
			}
			opt.unwrap()
		};

		if self.completed.contains(identifier) {
			return RelationshipProcess::Incomplete(self)
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
		let mut compatible_modules = modules_ksp_compatible.iter().filter(|module| &module.identifier == identifier).collect::<Vec<_>>();

		/* Handle virtual modules */
		{
			let is_virtual = compatible_modules.is_empty();
	
			if is_virtual {
				if !self.decisions.contains_key(identifier) {
					let providers = self.metadb.get_modules().iter()
						.filter(|module| module.provides.contains(identifier))
						.map(|module| module.identifier.clone())
						.collect::<HashSet<_>>();

					/* Handle no providers case. See comment above `compatible_modules` for more info */
					if providers.is_empty() {
						self.failed.push(FailedResolve::NoCompatibleKspVersion(identifier.clone()));
						return RelationshipProcess::Incomplete(self);
					}

					if providers.len() == 1 {
						let new_id = providers.iter().collect::<Vec<_>>()[0];
						compatible_modules = modules_ksp_compatible.iter().filter(|module| &module.identifier == new_id).collect::<Vec<_>>();
					} else {
						/* Check confirmed to see if decision has already been made */
						if self.confirmed.iter().any(|c| providers.contains(&c.identifier)) {
							self.resolve_queue.remove(0);
							return RelationshipProcess::Incomplete(self)
						} else {
							return RelationshipProcess::MultipleProviders(
								RelationshipMutlipleProvidersDecision {
									virtual_identifier: identifier.clone(),
									options: providers,
									selection: "".to_string(),
									inner: self,
								}
							)
						}
					}
				} else {
					let new_identifier = self.decisions.get(identifier).unwrap().clone();
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
			self.failed.push(FailedResolve::NoCompatibleCandidates(identifier.clone()));
			return RelationshipProcess::Incomplete(self);
		}

		RelationshipProcess::Incomplete(self)
	}

	fn add_decision(&mut self, virtual_identifier: String, identifier: String) {
		self.decisions.insert(virtual_identifier, identifier);
	}
}