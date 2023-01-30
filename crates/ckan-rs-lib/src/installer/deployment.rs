//! # Deployment
//! 
//! Here we link the files from the downloaded content into the game's directory.
//! 
//! Note that there is no utilities for operating on a single module at a time. 
//! this is because the hard links used in deployment are so cheap to create 
//! it's simply easier to redeploy the modules every time a change is made.
//! 

use std::collections::VecDeque;
use std::path::PathBuf;
use std::path::Path;

use crate::ModuleInfo;
use crate::metadb::ckan::ModUniqueIdentifier;
use crate::metadb::ckan::SourceDirective;

#[derive(Debug)]
pub enum DeploymentError {
	MissingContent,
	HardLink(std::io::Error),
	Copy(std::io::Error),
}

macro_rules! hardlink {
	($source:expr, $destination:expr) => {
		std::fs::hard_link($source, $destination).map_err(DeploymentError::HardLink)
	};
}

/// Deciphers the install directives into a simpler (`source`, `destination`) tuple.
/// where `source` is an absolute path and `destination` is relative to the game directory.
fn get_install_instructions(module: &ModuleInfo, extracted_archive: impl AsRef<Path>) -> Result<Vec<(PathBuf, PathBuf)>, std::io::Error> {
	let extracted_archive = extracted_archive.as_ref();

	let install_instructions = Vec::<(PathBuf, PathBuf)>::new();

	if module.install.is_empty() {
		/* "If no install sections are provided, a CKAN client must find 
		the top-most directory in the archive that matches the module identifier,
		and install that with a target of GameData." */
		/* Sounds like the `find` source directive? */
		todo!()
	}

	for directive in &module.install {
		let mut instruction: (PathBuf, PathBuf) = Default::default();

		instruction.1 = if directive.install_to == "GameRoot" {
			todo!("GameRoot install directive not yet supported")
		} else {
			PathBuf::from(&directive.install_to)
			/* TODO: Check if the path exists, is valid, traversal attempts */
		};

		match &directive.source {
			SourceDirective::File(s) => {
				instruction.0 = extracted_archive.join(PathBuf::from(&s));
				instruction.1 = instruction.1.join(PathBuf::from(&s).file_name().expect("weird paths in source directive"));
			},
			SourceDirective::Find(s) => {
				let mut queue = VecDeque::<PathBuf>::new();
				queue.push_back(extracted_archive.to_path_buf());
				'search: while let Some(p) = queue.pop_front() {
					for entry in p.read_dir()? {
						let entry = entry?;
						if entry.file_name().to_str().expect("filename isn't unicode") == s {
							instruction.0 = pathdiff::diff_paths(entry.path(), extracted_archive).unwrap();
							break 'search;
						}
						if entry.path().is_dir() {
							queue.push_back(entry.path());
							continue;
						}
					}
				}
			},
			SourceDirective::FindRegExp(s) => {
				/* TODO: Regex */
				todo!("FindRegExp not implemented yet!");
			},
		};
	}

	Ok(install_instructions)
}

/// Cleans the given instance of all modded files.
pub async fn clean_deployment(options: &crate::CkanRsOptions, instance: &mut crate::game_instance::GameInstance) -> Result<(), DeploymentError> {
	todo!();
	instance.tracked.clear();
}

/// Cleans the instance then links all required mod files.
pub async fn redeploy_modules(options: &crate::CkanRsOptions, db: crate::MetaDB, instance: &mut crate::game_instance::GameInstance) -> Result<(), DeploymentError> {
	clean_deployment(options, instance).await?;

	let mut tracked_files = Vec::<(&ModUniqueIdentifier, Vec<String>)>::new();
	
	for module in instance.get_enabled_modules() {
		let module = db.get_from_unique_id(module).expect("module no longer exists in metadb.");
		let path = super::content::get_module_deployment_path(options, &module.unique_id);
		let path = path.exists().then(|| path).ok_or(DeploymentError::MissingContent)?;

		let mut module_files = Vec::<String>::new();

		let install_instructions = get_install_instructions(module, path).unwrap();
	
		for (source, destination) in install_instructions {
			/* TODO: Install Methods */
			/* TODO: Handle directories */
			if source.is_dir() {
				todo!("directory source deployment")
			}
			hardlink!(&source, &destination)?;
			module_files.push(destination.to_string_lossy().to_string());
		}

		tracked_files.push((&module.unique_id, module_files));
	}

	for (module, files) in tracked_files {
		for f in files {
			instance.tracked.add_file(module, f);
		}
	}

	Ok(())
}