//! # CKAN's metadb
//! 
//! The metadb is composed of .ckan packages defined by a [specification](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)

pub mod ckan;
pub use ckan::Package;

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
	packages: HashSet<ckan::Package>,
}

impl MetaDB {
	/// Returns all packages in the database unfiltered and unsorted
	pub fn get_packages(&self) -> &HashSet<ckan::Package> {
		&self.packages
	}

	pub fn get_from_unique_id(&self, id: &ckan::PackageIdentifier) -> Option<&ckan::Package> {
		self.packages.iter().find(|package| package.identifier == *id)
	}

	pub fn get_from_identifier_and_version(&self, identifier: &str, version: &ckan::PackageVersion) -> Option<&ckan::Package> {
		let unique = ckan::PackageIdentifier {
			identifier: identifier.to_string(),
			version: version.clone(),
		};
		self.get_from_unique_id(&unique)
	}
}