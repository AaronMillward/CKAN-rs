use std::collections::HashSet;

use crate::metadb::ckan;

pub mod dep_graph_resolver;

pub mod dependencyresolver;
pub use dependencyresolver::RelationshipResolver;

/// Why a module was installed.
#[derive(Clone, PartialEq, Eq)]
enum InstallReason {
	AsDependency,
	Explicit,
}

/// How a version was selected.
#[derive(Clone, PartialEq, Eq)]
enum ModuleVersionReason {
	/// Version was specfically requested by the user
	Explicit,
	/// Version was deduced from the resolver
	Infered,
}

/// Info about why a module was installed.
#[derive(Clone)]
pub struct ModuleReason {
	identifier: ckan::ModUniqueIdentifier,
	install_reason: InstallReason,
	version_reason: ModuleVersionReason,
}

pub struct ProfileTransaction {
	add: Vec<ckan::ModuleDescriptor>,
	remove: Vec<ckan::ModuleDescriptor>,

	inner: Profile,
}

impl ProfileTransaction {
	pub fn new(profile: Profile) -> ProfileTransaction {
		Self {
			inner: profile,
			add: Default::default(),
			remove: Default::default(),
		}
	}

	pub fn add_modules(&mut self, modules: &[ckan::ModuleDescriptor]) {
		for m in modules {
			self.add.push(m.clone());
		}
	}

	pub fn remove_modules(&mut self, modules: &[ckan::ModuleDescriptor]) {
		for m in modules {
			self.remove.push(m.clone());
		}
	}

	pub fn commit(self) -> Profile {
		/* 
		 * 1. Check `add` and `remove` for contradicting descriptors 
		 * 2. Create a new list of modules by removing `remove` from explicitly installed
		 * 3. Join the `add` list to this new list
		 * 4. Run this list through the resolver
		 * 5. Diff the result with the existing list
		 * 6. Apply changes from diff
		 */

		let new_top_depends = {
			let mut new_top_depends = self.inner.installed_modules.iter()
				.filter(|m| m.install_reason == InstallReason::Explicit)
				.filter(|m| {
					for rem in &self.remove {
						if ckan::does_module_match_descriptor(&m.identifier, rem) {
							return false
						}
					}
					true
				})
				.cloned()
				.map(|r| ckan::ModuleDescriptor::new(
					r.identifier.identifier,
					match r.version_reason {
						ModuleVersionReason::Explicit => ckan::ModVersionBounds::Explicit(r.identifier.version),
						ModuleVersionReason::Infered => ckan::ModVersionBounds::Any,
					}
				))
				.collect::<Vec<_>>();
			
			new_top_depends.append(&mut self.add.clone());
			
			new_top_depends
		};

		// let mut resolver = RelationshipResolver::new(compatible_ksp_versions, requirements, &db);

		// loop {
		// 	let process = resolver.step();
		// 	match process {
		// 		RelationshipProcess::Incomplete => {},
		// 		RelationshipProcess::MultipleProviders(decision) => {
		// 			let mut options = decision.get_options().iter().collect::<Vec<_>>();
		// 			options.sort(); /* Sort to get a consistent result when testing */
		// 			let dec = options[0].clone();
		// 			eprintln!("Adding \"{}\" to decisions from list {:?}", dec, decision.get_options());
		// 			let d = match decision.select(dec) {
		// 				MutlipleProvidersDecisionValidation::Valid(d) => d,
		// 				MutlipleProvidersDecisionValidation::Invalid(_) => panic!("Invalid decision"),
		// 			};
		// 			resolver.add_decision(d);
		// 		},
		// 		RelationshipProcess::Halt => {
		// 			eprintln!("Resolver halted, printing failures:");
		// 			for fail in resolver.get_failed_resolves() {
		// 				match fail {
		// 					FailedResolve::ModulesConflict(l, r) => eprintln!("Conflict\n\t{:?}\n\t\t{:?}\n\t\t{:?}\n\t{:?}\n\t\t{:?}\n\t\t{:?}", &l.identifier, &l.version, &l.conflicts, &r.identifier, &r.version, &r.conflicts),
		// 					f => eprintln!("{:?}", f),
		// 				}
		// 			}
		// 			panic!("Resolver Halted");
		// 		},
		// 		RelationshipProcess::Complete => { break; },
		// 	}
		// }

		todo!();
	}

	pub fn cancel(self) -> Profile {
		self.inner
	}
}

pub struct Profile {
	pub compatible_ksp_versions: HashSet<ckan::KspVersion>,
	installed_modules: Vec<ModuleReason>,
}

impl Profile {
	pub fn start_transaction(self) -> ProfileTransaction {
		ProfileTransaction::new(self)
	}
}