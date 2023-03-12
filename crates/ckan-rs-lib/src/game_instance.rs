use crate::metadb::ckan;

pub mod filetracker;

#[derive(Debug)]
pub enum GameInstanceError {
	RequiredFilesMissing(std::io::Error),
}

#[derive(Debug)]
pub struct GameInstance {
	path: std::path::PathBuf,
	pub compatible_ksp_versions: Vec<ckan::KspVersion>,
	enabled_packages: Vec<ckan::PackageIdentifier>,
	pub tracked: filetracker::TrackedFiles,
	pub deployment_dir: std::path::PathBuf,
}

impl GameInstance {
	pub fn game_dir(&self) -> &std::path::Path {
		&self.path
	}

	pub fn is_file_installable(&self, path: String) -> bool {
		todo!()
	}

	pub fn get_enabled_packages(&self) -> &Vec<ckan::PackageIdentifier> {
		&self.enabled_packages
	}

	pub fn add_enabled_packages(&mut self, package: impl AsRef<ckan::PackageIdentifier>) {
		self.enabled_packages.push(package.as_ref().clone());
	}

	pub fn clear_enabled_packages(&mut self) {
		self.enabled_packages.clear();
	}

	pub fn new(game_root_directory: impl AsRef<std::path::Path>, deployment_dir: std::path::PathBuf) -> Result<GameInstance, GameInstanceError>{
		let game_root_directory = game_root_directory.as_ref();
		std::fs::metadata(game_root_directory).map_err(GameInstanceError::RequiredFilesMissing)?; // Gives the user more info compared to using `game_root_directory.exists()`
		
		let build_id_filepath = game_root_directory.join("buildID.txt");
		std::fs::metadata(build_id_filepath).map_err(GameInstanceError::RequiredFilesMissing)?;

		/* TODO: Get version from buildID */

		Ok(GameInstance {
			path: game_root_directory.to_path_buf(),
			compatible_ksp_versions: vec![ckan::KspVersion::try_from("1.12.3").unwrap()],
			tracked: Default::default(),
			enabled_packages: Default::default(),
			deployment_dir
		})
	}
}