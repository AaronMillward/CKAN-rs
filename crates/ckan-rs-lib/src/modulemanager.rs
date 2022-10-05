use std::collections::HashSet;

use crate::metadb::ckan;

pub mod dependencyresolver;

/// Why a module was installed.
enum InstallReason {
	AsDependency,
	Explicit,
}

/// How a version was selected.
enum ModuleVersionReason {
	/// Version was specfically requested by the user
	Explicit,
	/// Version was deduced from the resolver
	Infered,
}

/// Info about why a module was installed.
pub struct ModuleReason {
	identifier: String,
	version: ckan::ModVersion,
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
		/* TODO: */

		/* Check `add` and `remove` for contradicting descriptors */
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