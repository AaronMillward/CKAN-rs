use crate::metadb::{ckan, MetaDB};
use crate::relationship_resolver;

pub mod filetracker;
pub mod transaction;
use transaction::GameInstanceTransaction;

/* TODO: Install Reason */

pub struct GameInstance {
	pub compatible_ksp_versions: Vec<ckan::KspVersion>,
	wanted: Vec<relationship_resolver::InstallRequirement>,
	tracked: filetracker::TrackedFiles,
}

impl GameInstance {
	pub fn start_transaction(self, metadb: &MetaDB) -> GameInstanceTransaction {
		GameInstanceTransaction::new(self, metadb)
	}
}