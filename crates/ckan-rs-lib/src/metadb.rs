//! # CKAN's MetaDB, a repository of KSP mods.
//! 
//! The metadb is composed of `.ckan` packages defined by a [specification](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)

pub mod package;

mod generation;
pub use generation::generate_latest;

mod iterator;
pub use iterator::KspVersionMatchesExt;
pub use iterator::DescriptorMatchesExt;
pub use iterator::GetProvidersExt;
pub use iterator::ModVersionMatchesExt;

use std::collections::{HashSet, HashMap};
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use package::{Package, PackageIdentifier, PackageVersion};

/// A list of build numbers found in `buildID.txt` and their associated version strings.
pub type BuildIDList = HashMap<i32, String>;

/// A database of packages that can be installed to the game.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetaDB {
	packages: HashSet<Package>,
	/* TODO: Store builds field seperately so we don't have to load the whole DB to get game versions. */
	builds: BuildIDList,
}

impl MetaDB {
	/// Returns all packages in the database unfiltered and unsorted
	pub fn get_packages(&self) -> &HashSet<Package> {
		&self.packages
	}

	pub fn get_from_unique_id(&self, id: impl AsRef<PackageIdentifier>) -> Option<&Package> {
		self.packages.iter().find(|package| package.identifier == *id.as_ref())
	}

	pub fn get_from_identifier_and_version(&self, identifier: &str, version: &PackageVersion) -> Option<&Package> {
		let unique = PackageIdentifier {
			identifier: identifier.to_string(),
			version: version.clone(),
		};
		self.get_from_unique_id(unique)
	}

	pub fn get_game_builds(&self) -> &BuildIDList {
		&self.builds
	}

	/// Loads the MetaDB.
	/// 
	/// # Errors
	/// - [`IO`](`crate::error::Error::IO`) when opening or reading from the file.
	/// - [`Parse`](`crate::error::Error::Parse`) when deserializing the file, this is likely due to the DB format changing and so needs regenerating.
	pub fn load_from_disk(config: &crate::CkanRsConfig) -> crate::Result<MetaDB> {
		let path = config.data_dir().join("metadb.bin");
		let mut f = std::fs::File::open(path)?;
		let mut v = Vec::<u8>::new();
		f.read_to_end(&mut v)?;
		bincode::deserialize::<MetaDB>(&v).map_err(|_| crate::error::Error::Parse("Deserialize failed".to_string()))
	}

	/// Saves the MetaDB.
	/// 
	/// # Errors
	/// - [`IO`](`crate::error::Error::IO`) when creating or writing to the file.
	/// - [`Parse`](`crate::error::Error::Parse`) when serializing the file.
	pub fn save_to_disk(&self, config: &crate::CkanRsConfig) -> crate::Result<()> {
		let path = config.data_dir().join("metadb.bin");
		let data = bincode::serialize(self).map_err(|_| crate::error::Error::Parse("Serialize failed".to_string()))?;
		let mut f = std::fs::File::create(path)?;
		f.write_all(&data)?;
		Ok(())
	}
}