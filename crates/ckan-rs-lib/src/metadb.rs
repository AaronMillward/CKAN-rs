//! # CKAN's metadb
//! 
//! The metadb is composed of .ckan modules defined by a [specification](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)

pub mod ckan;
pub use ckan::ModuleInfo;

mod generation;
pub use generation::get_latest_archive;

mod iterator;
pub use iterator::KspVersionMatchesExt;
pub use iterator::DescriptorMatchesExt;
pub use iterator::GetProvidersExt;
pub use iterator::ModVersionMatchesExt;

use std::collections::HashSet;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaDB {
	modules: HashSet<ckan::ModuleInfo>,
}

impl MetaDB {
	/// Returns all modules in the database unfiltered and unsorted
	pub fn get_modules(&self) -> &HashSet<ckan::ModuleInfo> {
		&self.modules
	}

	pub fn get_from_identifier_and_version(&self, identifier: &str, version: &ckan::ModVersion) -> Option<&ckan::ModuleInfo> {
		let unique = ckan::ModUniqueIdentifier {
			identifier: identifier.to_string(),
			version: version.clone(),
		};
		self.modules.iter().find(|module| module.unique_id == unique)
	}
}