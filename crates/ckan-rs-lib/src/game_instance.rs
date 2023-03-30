use std::{collections::HashSet, io::Read};

use crate::metadb::package;

pub mod filetracker;

/// A single install (instance) of a game.
#[derive(Debug)]
pub struct GameInstance {
	path: std::path::PathBuf,
	compatible_ksp_versions: Vec<package::KspVersion>,
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
	pub fn new(builds: &std::collections::HashMap<i32, String>, game_root_directory: impl AsRef<std::path::Path>, deployment_dir: std::path::PathBuf) -> crate::Result<GameInstance>{
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
				vec![package::KspVersion::try_from(s.as_str()).expect("builds.json contains invalid ksp version.")]
			} else {
				log::error!("builds.json missing build id {}, try updating metadb.", id);
				return Err(crate::Error::Parse(format!("builds.json missing build id {}, try updating metadb.", id)))
			}
		};
		
		log::info!("Created new game instance at path {}", game_root_directory.display());
		
		Ok(GameInstance {
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

	pub fn set_compatible_ksp_versions(&mut self, value: Vec<package::KspVersion>) {
		self.compatible_ksp_versions = value;
	}

	pub fn get_compatible_ksp_versions(&self) -> &Vec<package::KspVersion> {
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

	/* TODO: Serialization */
}

/* TODO: Instance List */