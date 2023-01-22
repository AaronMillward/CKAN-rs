use crate::metadb::{ckan, MetaDB};
use crate::relationship_resolver;

pub mod filetracker;
pub mod transaction;
pub use transaction::GameInstanceTransaction;

/* TODO: Install Reason */

pub struct GameInstance {
	path: std::path::PathBuf,
	pub compatible_ksp_versions: Vec<ckan::KspVersion>,
	wanted_modules: Vec<relationship_resolver::InstallRequirement>,
	pub tracked: filetracker::TrackedFiles,
}

impl GameInstance {
	pub fn game_dir(&self) -> &std::path::Path {
		&self.path
	}


	pub fn start_transaction(self, metadb: &MetaDB) -> GameInstanceTransaction {
		GameInstanceTransaction::new(self, metadb)
	}

	pub fn is_file_installable(&self, path: String) -> bool {
		todo!()
	}
}