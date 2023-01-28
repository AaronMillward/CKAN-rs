use crate::metadb::{ckan, MetaDB};
use crate::relationship_resolver;

pub mod filetracker;
pub mod transaction;
pub use transaction::GameInstanceTransaction;

use self::filetracker::TrackedFiles;

pub enum GameInstanceError {
	RequiredFilesMissing(std::io::Error),
}

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

	pub fn new(game_root_directory: impl AsRef<std::path::Path>) -> Result<GameInstance, GameInstanceError>{
		let game_root_directory = game_root_directory.as_ref();
		std::fs::metadata(game_root_directory).map_err(GameInstanceError::RequiredFilesMissing)?; // Gives the user more info compared to using `game_root_directory.exists()`
		
		let build_id_filepath = game_root_directory.join("buildID.txt");
		std::fs::metadata(build_id_filepath).map_err(GameInstanceError::RequiredFilesMissing)?;

		/* TODO: Get version from buildID */

		Ok(GameInstance {
			path: game_root_directory.to_path_buf(),
			compatible_ksp_versions: vec![ckan::KspVersion::try_from("1.12.3").unwrap()],
			tracked: Default::default(),
			wanted_modules: Default::default(),
		})
	}
}