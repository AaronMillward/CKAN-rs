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
use crate::metadb::ckan::InstallDirective;
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

/// Deciphers the install directives into a simpler (`source`, `destination`) tuple.
/// where `source` is an absolute path and `destination` is relative to the game directory.
fn get_install_instructions(package: &Package, extracted_archive: impl AsRef<Path>) -> Result<Vec<(PathBuf, PathBuf)>, std::io::Error> {
	fn process_directive(directive: &InstallDirective, extracted_archive: &Path) -> Vec<(PathBuf, PathBuf)> {
		let mut instructions: Vec<(PathBuf, PathBuf)> = Default::default();

		let destination = if directive.install_to == "GameRoot" {
			PathBuf::from("")
		} else {
			PathBuf::from(&directive.install_to)
			/* TODO: Check if the path exists, is valid, traversal attempts */
		};

		let find_matches_files = directive.additional.iter().any(|e| matches!(e, crate::metadb::ckan::OptionalDirective::FindMatchesFiles(x) if *x));

		/* TODO: Other optional directives */

		match &directive.source {
			SourceDirective::File(s) => {
				instructions.push((
						extracted_archive.join(PathBuf::from(&s)),
						destination.join(PathBuf::from(&s).file_name().expect("weird paths in source directive"))
				));
			},
			SourceDirective::Find(s) => {
				for entry in walkdir::WalkDir::new(extracted_archive).into_iter() {
					let entry = entry.unwrap().into_path();
					if entry.is_file() && !find_matches_files { continue; }
					if entry.file_name().expect("filepath ends in \"..\"").to_str().expect("filename isn't unicode") == s {
						if entry.is_file() {
							instructions.push((
								entry,
								destination.join(s)
							));
						} else if entry.is_dir() {
							for entry2 in walkdir::WalkDir::new(&entry).into_iter() {
								let entry2 = entry2.unwrap().into_path();
								if entry2.is_file() {
									instructions.push((
										entry2.clone(),
										destination.join(pathdiff::diff_paths(&entry2, &entry.with_file_name("")).unwrap())
									));
								}
							}
						}
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
						instructions.push((
							entry.clone(),
							destination.join(entry.file_name().unwrap()),
						));
						break
					}
				}
			},
		};

		instructions
	}

	let extracted_archive = extracted_archive.as_ref();

	let mut install_instructions = Vec::<(PathBuf, PathBuf)>::new();

	let directives = if package.install.is_empty() {
		 /* "If no install sections are provided, a CKAN client must find 
		 the top-most directory in the archive that matches the module identifier,
		 and install that with a target of GameData." */
		 /* Sounds like the `find` source directive? */
		std::borrow::Cow::Owned(vec![InstallDirective::new(
			SourceDirective::Find(package.identifier.identifier.clone()),
			"GameData".to_string(),
			Default::default()
		)])
	} else {
		std::borrow::Cow::Borrowed(&package.install)
	};

	for directive in directives.iter() {
		install_instructions.append(&mut process_directive(directive, extracted_archive))
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
		let path = path.exists().then_some(path).ok_or(DeploymentError::MissingContent)?;

		let mut package_files = Vec::<String>::new();

		let install_instructions = get_install_instructions(package, path).unwrap();
	
		for (source, destination) in install_instructions {
			/* TODO: Install Methods */
				let final_destination = instance.game_dir().join(&destination);
				std::fs::create_dir_all(&final_destination.with_file_name("")).map_err(DeploymentError::CreateDirectory)?;
				std::fs::hard_link(&source, &final_destination).map_err(DeploymentError::HardLink).expect("hardlink failed.");
				package_files.push(destination.to_string_lossy().to_string());
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