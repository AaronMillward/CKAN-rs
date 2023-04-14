use std::{collections::HashSet, io::{Read, Write}, path::Path};

use crate::metadb::package;

pub mod filetracker;

/// A single install (instance) of a game.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameInstance {
	name: String,
	path: std::path::PathBuf,
	compatible_ksp_versions: Vec<package::KspVersionReal>,
	enabled_packages: HashSet<package::PackageIdentifier>,
	pub tracked: filetracker::TrackedFiles,
	pub deployment_dir: std::path::PathBuf,
}

impl GameInstance {
	/// Creates a new instance.
	/// 
	/// # Errors
	/// - [`crate::error::Error::IO`] when the directory is invalid.
	/// - [`crate::error::Error::Parse`] when extracting the build id from `buildID.txt`.
	pub fn new(config: &crate::CkanRsConfig, builds: &std::collections::HashMap<i32, String>, name: String, game_root_directory: impl AsRef<std::path::Path>, deployment_dir: std::path::PathBuf) -> crate::Result<GameInstance> {
		let instances_dir = config.data_dir().join("instances");
		if !instances_dir.exists() {
			std::fs::create_dir_all(&instances_dir)?;
		}

		log::debug!("Checking for existing instances in {}", instances_dir.display());

		for instance_path in instances_dir.read_dir()?.map(|r| r.map(|r| r.path())) {
			let instance = GameInstance::load_by_file(instance_path?)?;
			let instance_name_taken = instance.name == name;
			let game_root_in_use = instance.game_dir() == game_root_directory.as_ref();
			if instance_name_taken || game_root_in_use {
				return Err(crate::Error::AlreadyExists)
			}
		}

		log::debug!("Checking validity of game root {}", game_root_directory.as_ref().display());

		let game_root_directory = game_root_directory.as_ref();
		std::fs::metadata(game_root_directory)?; // Gives the user more info compared to using `game_root_directory.exists()`
		
		let build_id_filepath = game_root_directory.join("buildID.txt");
		std::fs::metadata(&build_id_filepath)?;

		/* Get version from buildID */ 
		let compatible_ksp_versions = {
			let mut file = std::fs::File::open(build_id_filepath)?;
			let mut s = String::default();
			file.read_to_string(&mut s)?;
			let mut id = None;
			for line in s.lines() {
				if line.starts_with("build id =") {
					let buildid = &line[12..];
					id = Some(buildid.parse::<i32>().map_err(|_| crate::Error::Parse(format!("Couldn't parse \"{}\" to an int", buildid)))?);
				}
			}
			
			let id = id.ok_or_else(|| crate::Error::Parse("Build ID not found in buildID.txt".to_string()))?;
			
			if let Some(s) = builds.get(&id) {
				vec![package::KspVersionReal::try_from(s.as_str()).expect("builds.json ksp version string should be valid.")]
			} else {
				return Err(crate::Error::Parse(format!("builds.json missing build id {}, try updating metadb.", id)))
			}
		};
		
		log::info!("Created new game instance at path {}", game_root_directory.display());
		
		Ok(GameInstance {
			name,
			path: game_root_directory.to_path_buf(),
			compatible_ksp_versions,
			tracked: Default::default(),
			enabled_packages: Default::default(),
			deployment_dir
		})
	}

	/* Fields */

	pub fn game_dir(&self) -> &std::path::Path {
		&self.path
	}

	pub fn set_compatible_ksp_versions(&mut self, value: Vec<package::KspVersionReal>) {
		self.compatible_ksp_versions = value;
	}

	pub fn get_compatible_ksp_versions(&self) -> &Vec<package::KspVersionReal> {
		&self.compatible_ksp_versions
	}

	/* Package Management */

	pub fn get_enabled_packages(&self) -> &HashSet<package::PackageIdentifier> {
		&self.enabled_packages
	}

	/// Enables a given package so that it is deployed the next time [`crate::installer::deployment::redeploy_packages`] is called.
	pub fn enable_package(&mut self, package: impl AsRef<package::PackageIdentifier>) {
		log::trace!("Enabling package {} on instance at {}", package.as_ref(), self.game_dir().display());
		self.enabled_packages.insert(package.as_ref().clone());
	}

	/// Disables a given package so that it is not deployed the next time [`crate::installer::deployment::redeploy_packages`] is called.
	pub fn disable_package(&mut self, package: impl AsRef<package::PackageIdentifier>) {
		log::trace!("Disabling package {} on instance at {}", package.as_ref(), self.game_dir().display());
		self.enabled_packages.remove(package.as_ref());
	}

	pub fn clear_enabled_packages(&mut self) {
		log::trace!("Clearing enabled packages on instance at {}", self.game_dir().display());
		self.enabled_packages.clear();
	}

	/* Serialization */

	/// Loads an instance with the given name.
	/// 
	/// # Errors
	/// - [`crate::error::Error::IO`] when opening or reading from the file.
	/// - [`crate::error::Error::SerdeJSON`] when deserializing the file.
	pub fn load_by_name(config: &crate::CkanRsConfig, name: impl AsRef<str>) -> crate::Result<Self> {
		let path = config.data_dir().join("instances").join(format!("{}.json", name.as_ref()));
		Self::load_by_file(path)
	}

	/// Loads an instance with from a file at a given path.
	/// 
	/// # Errors
	/// - [`crate::error::Error::IO`] when opening or reading from the file.
	/// - [`crate::error::Error::SerdeJSON`] when deserializing the file.
	fn load_by_file(path: impl AsRef<Path>) -> crate::Result<Self> {
		let mut file = std::fs::File::open(path)?;
		let mut s: String = Default::default();
		file.read_to_string(&mut s)?;
		Ok(serde_json::from_str(&s)?)
	}

	/// Saves the instance to a JSON file.
	/// 
	/// # Errors
	/// - [`crate::error::Error::IO`] when opening the file, writing to it or creating it's parent directories.
	/// - [`crate::error::Error::SerdeJSON`] when serializing the file.
	pub fn save_to_disk(&self, config: &crate::CkanRsConfig) -> crate::Result<()> {
		let path = config.data_dir().join("instances").join(format!("{}.json", self.name));
		std::fs::create_dir_all(path.with_file_name(""))?;
		let json = serde_json::to_string_pretty(self)?;
		let mut file = std::fs::File::create(path)?;
		file.write_all(json.as_bytes())?;
		Ok(())
	}
}