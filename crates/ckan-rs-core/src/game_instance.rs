//! Game installation handling.

use std::{io::{Read, Write}, path::Path, collections::HashSet};

use crate::metadb::package;
use crate::metadb::package::KspVersionReal;

use crate::relationship_resolver::PackageTree;
use crate::relationship_resolver::{Complete, InProgress};

pub mod filetracker;

/// A single instance of a game.
/// 
/// This struct is saved to a JSON file in the ckan-rs data directory.
/// 
/// Instances are named so they can be loaded by that name.
/// 
/// It is recomended after each operation (enabling/disabling/redeploying) to call
/// [`save_to_disk()`](GameInstance::save_to_disk()) as this is not done automatically.
/// If changes are made to an instance but not tracked due to the instance not being
/// saved it could require manual intervention to fix.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameInstance {
	name: String,
	path: std::path::PathBuf,
	compatible_ksp_versions: Vec<KspVersionReal>,
	package_tree: PackageTree<Complete>,
	pub tracked: filetracker::TrackedFiles,
	pub deployment_dir: std::path::PathBuf,
}

impl GameInstance {
	/// Creates a new instance.
	/// 
	/// # Parameters
	/// - `builds` - A list of KSP build numbers and their corisponding version string. Usually sourced from [`get_game_builds()`](crate::metadb::MetaDB::get_game_builds())
	/// - `name` - The identifier CKAN-rs will use to track this instance.
	/// - `game_root_directory` - The path to the root of the game install. this is where the KSP executable is located.
	/// - `deployment_dir` - This is where modded files will be installed to before being linked to the games directory.
	/// due to the use of hard links this directory must be on the same drive as `game_root_directory`.
	/// # Errors
	/// - [`IO`](crate::error::Error::IO) when the directory is invalid.
	/// - [`Parse`](crate::error::Error::Parse) when extracting the build id from `buildID.txt`.
	pub fn new(config: &crate::Config, builds: &crate::metadb::BuildIDList, name: String, game_root_directory: impl AsRef<std::path::Path>, deployment_dir: std::path::PathBuf) -> crate::Result<GameInstance> {
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
				vec![KspVersionReal::try_from(s.as_str()).expect("builds.json ksp version string should be valid.")]
			} else {
				return Err(crate::Error::Parse(format!("builds.json missing build id {}, try updating metadb.", id)))
			}
		};

		let package_tree = PackageTree::<Complete>::new(compatible_ksp_versions.clone());
		
		log::info!("Created new game instance at path {}", game_root_directory.display());
		
		Ok(GameInstance {
			name,
			path: game_root_directory.to_path_buf(),
			compatible_ksp_versions,
			tracked: Default::default(),
			package_tree,
			deployment_dir
		})
	}

	/* Fields */

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn path(&self) -> &std::path::Path {
		&self.path
	}

	pub fn game_dir(&self) -> &std::path::Path {
		&self.path
	}

	pub fn set_compatible_ksp_versions(&mut self, value: Vec<KspVersionReal>) {
		self.compatible_ksp_versions = value;
	}

	pub fn compatible_ksp_versions(&self) -> &Vec<KspVersionReal> {
		&self.compatible_ksp_versions
	}

	/* Package Management */

	pub fn enabled_packages(&self) -> Vec<package::PackageIdentifier> {
		self.package_tree.get_all_packages()
	}

	pub fn alter_package_requirements<F>(
		&mut self,
		metadb: &crate::MetaDB,
		add: impl IntoIterator<Item = crate::relationship_resolver::InstallTarget>,
		remove: impl IntoIterator<Item = crate::relationship_resolver::InstallTarget>,
		decision_handler: F) 
		-> Result<(Vec<crate::metadb::package::PackageIdentifier>, Vec<crate::metadb::package::PackageIdentifier>), ()>
	where F: Fn(&mut PackageTree<InProgress>, Vec<crate::relationship_resolver::DecisionInfo>),
	{
		use crate::relationship_resolver::ResolverStatus;
		let mut pt = self.package_tree.clone().alter_package_requirements(add, remove);
		loop {
			match pt.attempt_resolve(metadb) {
				ResolverStatus::Complete => {
					log::info!("Clearing loose packages after complete resolve.");
					// pt.clear_loose_packages();
					let pt = pt.complete().expect("resolve` should be complete if sending `complete` status.");

					let new_package_list: HashSet<_> = pt.get_all_packages().into_iter().collect();
					let prev_package_list: HashSet<_> = self.package_tree.get_all_packages().into_iter().collect();

					let removed: Vec<_> = prev_package_list.difference(&new_package_list).cloned().collect();
					let added: Vec<_> = new_package_list.difference(&prev_package_list).cloned().collect();

					self.package_tree = pt;

					return Ok((added,removed))
				},
				ResolverStatus::DecisionsRequired(infos) => {
					decision_handler(&mut pt, infos)
				},
				ResolverStatus::Failed(fails) => {
					for f in fails {
						log::error!("Resolver failed on package {} with error: {}", f.0, f.1);
					}
					return Err(())
				},
			}
		}
	}

	/// Disables all packages so they are not deployed the next time [`redeploy_packages()`](GameInstance::redeploy_packages()) is called.
	pub fn clear_enabled_packages(&mut self) {
		log::trace!("Clearing enabled packages on instance at {}", self.game_dir().display());
		self.package_tree.clear_all_packages();
	}

	/* Serialization */

	/// Loads an instance with the given name.
	/// 
	/// # Errors
	/// - [`IO`](crate::error::Error::IO) when opening or reading from the file.
	/// - [`SerdeJSON`](crate::error::Error::SerdeJSON) when deserializing the file.
	pub fn load_by_name(config: &crate::Config, name: impl AsRef<str>) -> crate::Result<Self> {
		let path = config.data_dir().join("instances").join(format!("{}.json", name.as_ref()));
		Self::load_by_file(path)
	}

	/// Loads an instance from a file at a given path.
	/// 
	/// # Errors
	/// - [`IO`](crate::error::Error::IO) when opening or reading from the file.
	/// - [`SerdeJSON`](crate::error::Error::SerdeJSON) when deserializing the file.
	pub fn load_by_file(path: impl AsRef<Path>) -> crate::Result<Self> {
		let file = std::fs::File::open(path)?;
		Ok(bincode::deserialize_from(file)?)
	}

	/// Saves the instance to a JSON file.
	/// 
	/// # Errors
	/// - [`IO`](crate::error::Error::IO) when opening the file, writing to it or creating it's parent directories.
	/// - [`Bincode`](crate::error::Error::Bincode) when serializing the file.
	pub fn save_to_disk(&self, config: &crate::Config) -> crate::Result<()> {
		let path = config.data_dir().join("instances").join(format!("{}.json", self.name));
		std::fs::create_dir_all(path.with_file_name(""))?;
		let file = std::fs::File::create(path)?;
		bincode::serialize_into(file, self)?;
		Ok(())
	}
}