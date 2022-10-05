//! # CKAN's metadb
//! 
//! The metadb is composed of .ckan modules defined by a [specification](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)

pub mod ckan;

mod generation;
pub use generation::get_latest_archive;

mod iterator;
pub use iterator::KspVersionMatchesExt;
pub use iterator::DescriptorMatchesExt;

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

	pub fn get_from_identifier_and_version(&self, identifier: &str, version: &ckan::ModVersion) -> Option<&ckan::Ckan> {
		self.modules.iter().find(|module| module.identifier == identifier && &module.version == version)
	}
}