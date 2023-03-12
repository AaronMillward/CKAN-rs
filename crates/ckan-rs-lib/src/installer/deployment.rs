//! # Deployment
//! 
//! Here we link the files from the downloaded content into the game's directory.
//! 
//! Note that there is no utilities for operating on a single package at a time. 
//! this is because the hard links used in deployment are so cheap to create 
//! it's simply easier to redeploy the packages every time a change is made.
//! 

use std::path::PathBuf;
use std::path::Path;

use crate::Package;
use crate::metadb::ckan::PackageIdentifier;
use crate::metadb::ckan::SourceDirective;

#[derive(Debug)]
pub enum DeploymentError {
	MissingContent,
	HardLink(std::io::Error),
	Copy(std::io::Error),
	RemoveFailed(std::io::Error),
	CreateDirectory(std::io::Error),
	TraverseFailed,
}

macro_rules! hardlink {
	($source:expr, $destination:expr) => {
		std::fs::hard_link($source, $destination).map_err(DeploymentError::HardLink)
	};
}

/// Deciphers the install directives into a simpler (`source`, `destination`) tuple.
/// where `source` is an absolute path and `destination` is relative to the game directory.
fn get_install_instructions(package: &Package, extracted_archive: impl AsRef<Path>) -> Result<Vec<(PathBuf, PathBuf)>, std::io::Error> {
	let extracted_archive = extracted_archive.as_ref();

	let mut install_instructions = Vec::<(PathBuf, PathBuf)>::new();

	if package.install.is_empty() {
		/* "If no install sections are provided, a CKAN client must find 
		the top-most directory in the archive that matches the module identifier,
		and install that with a target of GameData." */
		/* Sounds like the `find` source directive? */
		todo!()
	}

	for directive in &package.install {
		let mut instruction: (PathBuf, PathBuf) = Default::default();

		instruction.1 = if directive.install_to == "GameRoot" {
			todo!("GameRoot install directive not yet supported")
		} else {
			PathBuf::from(&directive.install_to)
			/* TODO: Check if the path exists, is valid, traversal attempts */
		};

		let find_matches_files = directive.additional.iter().any(|e| matches!(e, crate::metadb::ckan::OptionalDirective::FindMatchesFiles(x) if *x));

		match &directive.source {
			SourceDirective::File(s) => {
				instruction.0 = extracted_archive.join(PathBuf::from(&s));
				instruction.1 = instruction.1.join(PathBuf::from(&s).file_name().expect("weird paths in source directive"));
			},
			SourceDirective::Find(s) => {
				for entry in walkdir::WalkDir::new(extracted_archive).into_iter() {
					let entry = entry.expect("failed to get file entry for source directive find.").into_path();
					if entry.is_file() && !find_matches_files { continue; }
					if entry.file_name().expect("filepath ends in \"..\"").to_str().expect("filename isn't unicode") == s {
						instruction.0 = entry;
						break
					}
				}
			},
			SourceDirective::FindRegExp(s) => {
				let regex = regex::Regex::new(s).expect("regex failed to compile.");

				for entry in walkdir::WalkDir::new(extracted_archive).into_iter() {
					let entry = entry.expect("failed to get file entry for source directive find.").into_path();
					let path = pathdiff::diff_paths(&entry, extracted_archive).expect("pathdiff failed.");
					if entry.is_file() && !find_matches_files { continue; }
					if regex.is_match(&path.to_string_lossy()) {
						instruction.1 = instruction.1.join(entry.file_name().unwrap());
						instruction.0 = entry;
						break
					}
				}
			},
		};

		install_instructions.push(instruction)
	}

	Ok(install_instructions)
}

/// Cleans the given instance of all package files.
pub async fn clean_deployment(instance: &mut crate::game_instance::GameInstance) -> Result<(), DeploymentError> {
	for f in instance.tracked.get_all_files() {
		let path = instance.game_dir().join(f);
		if path.exists() {
			std::fs::remove_file(path).map_err(DeploymentError::RemoveFailed)?;
			/* TODO: Clean empty directories */
		}
	}

	instance.tracked.clear();

	Ok(())
}

/// Cleans the instance then links all required package files.
pub async fn redeploy_packages(db: crate::MetaDB, instance: &mut crate::game_instance::GameInstance) -> Result<(), DeploymentError> {
	clean_deployment(instance).await?;

	let mut tracked_files = Vec::<(&PackageIdentifier, Vec<String>)>::new();
	
	for package in instance.get_enabled_packages() {
		let package = db.get_from_unique_id(package).expect("package no longer exists in metadb.");
		let path = instance.get_package_deployment_path(package);
		let path = path.exists().then(|| path).ok_or(DeploymentError::MissingContent)?;

		let mut package_files = Vec::<String>::new();

		let install_instructions = get_install_instructions(package, path).unwrap();
	
		for (source, destination) in install_instructions {
			/* TODO: Install Methods */
			for entry in walkdir::WalkDir::new(&source).into_iter() {
				let entry = entry.map_err(|_| DeploymentError::TraverseFailed)?.into_path();
				if entry.is_file() {
					let final_destination = instance.game_dir().join(&destination);
					std::fs::create_dir_all(&final_destination.with_file_name("")).map_err(DeploymentError::CreateDirectory)?;
					hardlink!(&entry, &final_destination).expect("hardlink failed.");
					package_files.push(destination.to_string_lossy().to_string());
				}
			}
		}

		tracked_files.push((&package.identifier, package_files));
	}

	for (package, files) in tracked_files {
		for f in files {
			instance.tracked.add_file(package, f);
		}
	}

	Ok(())
}