//! # CKAN's metadb
//! 
//! The metadb is composed of .ckan packages defined by a [specification](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)

pub mod package;
pub use package::Package;

mod generation;
pub use generation::generate_latest;

mod iterator;
pub use iterator::KspVersionMatchesExt;
pub use iterator::DescriptorMatchesExt;
pub use iterator::GetProvidersExt;
pub use iterator::ModVersionMatchesExt;

use std::collections::HashSet;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

/// A database of packages that can be installed to the game.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaDB {
	packages: HashSet<package::Package>,
}

impl MetaDB {
	/// Returns all packages in the database unfiltered and unsorted
	pub fn get_packages(&self) -> &HashSet<package::Package> {
		&self.packages
	}

	pub fn get_from_unique_id(&self, id: impl AsRef<package::PackageIdentifier>) -> Option<&package::Package> {
		self.packages.iter().find(|package| package.identifier == *id.as_ref())
	}

	pub fn get_from_identifier_and_version(&self, identifier: &str, version: &package::PackageVersion) -> Option<&package::Package> {
		let unique = package::PackageIdentifier {
			identifier: identifier.to_string(),
			version: version.clone(),
		};
		self.get_from_unique_id(unique)
	}

	pub fn load_from_disk(config: &crate::CkanRsConfig) -> crate::Result<MetaDB> {
		let path = config.data_dir().join("metadb.bin");
		let mut f = std::fs::File::open(path)?;
		let mut v = Vec::<u8>::new();
		f.read_to_end(&mut v).unwrap();
		bincode::deserialize::<MetaDB>(&v).map_err(|_| crate::error::Error::Parse("Deserialize failed".to_string()))
	}

	pub fn save_to_disk(&self, config: &crate::CkanRsConfig) -> crate::Result<()> {
		let path = config.data_dir().join("metadb.bin");
		let data = bincode::serialize(self).map_err(|_| crate::error::Error::Parse("Serialize failed".to_string()))?;
		let mut f = std::fs::File::create(path)?;
		f.write_all(&data)?;
		Ok(())
	}
}