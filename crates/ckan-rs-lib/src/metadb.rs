//! # CKAN's metadb
//! 
//! To improve performance the metadb is converted from it's native format, a series of JSON files to an sqlite database.
//! 
//! 
//! 

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
	pub fn get_modules(&self) -> &HashSet<ckan::Ckan> {
		&self.modules
	}

	pub fn get_from_identifier_and_version(&self, identifier: &str, version: &ckan::ModVersion) -> Option<&ckan::Ckan> {
		self.modules.iter()
			.filter(|module| module.identifier == identifier && &module.version == version)
			.collect::<Vec<_>>()
			.get(0)
			.copied()
	}
}