//! # CKAN's metadb
//! 
//! The metadb is composed of .ckan modules defined by a [specification](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)

pub mod ckan;

mod generation;
pub use generation::get_latest_archive;

use std::collections::HashSet;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaDB {
	modules: HashSet<ckan::Ckan>,
}

impl MetaDB {
	/// Returns all modules in the database unfiltered and unsorted
	pub fn get_modules(&self) -> &HashSet<ckan::Ckan> {
		&self.modules
	}

	/* XXX: Maybe this should be a method on an iterator */
	pub fn get_modules_matching_ksp_versions(&self, match_versions: &[ckan::KspVersion]) -> Vec<&ckan::Ckan> {
		self.get_modules().iter().filter(|module| {
			/* TODO: ksp_version_strict | this needs to be fixed in KspVersion */
			if let Some(version) = &module.ksp_version {
				if version.is_any() { return true }
				return match_versions.iter().any(|ksp| ckan::KspVersion::is_sub_version(ksp, version))
			}
			match (&module.ksp_version_min, &module.ksp_version_max) {
				(None, None) => {
					true /* XXX: at this point we can deduce the module has no ksp version requirements. The spec says this means it is "any" */
				},
				(None, Some(max))      => { match_versions.iter().any(|ksp| ksp <= max) },
				(Some(min), None)      => { match_versions.iter().any(|ksp| ksp >= min) },
				(Some(min), Some(max)) => { match_versions.iter().any(|ksp| min <= ksp && ksp <= max) }
			}
		})
		.collect::<Vec<_>>()
	}

	/* XXX: Maybe this should be a method on an iterator */
	/// Gets all modules that match a given descriptor including `provides` relationships.
	/// This means the output may not be all the same identifier
	pub fn get_modules_matching_descriptor(&self, relation: &ckan::ModuleDescriptor) -> Vec<&ckan::Ckan> {
		self.modules.iter()
		.filter(|module| {
			ckan::does_module_match_descriptor(module, relation)
		})
		.collect::<Vec<_>>()
	}

	pub fn get_from_identifier_and_version(&self, identifier: &str, version: &ckan::ModVersion) -> Option<&ckan::Ckan> {
		self.modules.iter().find(|module| module.identifier == identifier && &module.version == version)
	}
}