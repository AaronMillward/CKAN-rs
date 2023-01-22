//! Handles installing modules to a game directory

/* 
I quickly searched through the metadb using vim and it seems like the *vast* majority of content types are `application/zip`
so we're just going to consider them all zips and error otherwise for now.
 */

use std::collections::VecDeque;
use std::path::PathBuf;
use std::path::Path;

use crate::ModuleInfo;
use crate::metadb::ckan::SourceDirective;

pub mod download;
pub mod content;

#[derive(Debug)]
pub enum InstallError {
	MissingContent,
	HardLink(std::io::Error),
	Copy(std::io::Error),
}

fn hardlink(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<(), InstallError> {
	std::fs::hard_link(source, destination).map_err(InstallError::HardLink)
}

fn copy(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<(), InstallError> {
	std::fs::copy(source, destination).map(|_|()).map_err(InstallError::Copy)
}

pub async fn install_module(options: &crate::CkanRsOptions, instance: &mut crate::game_instance::GameInstance, module: &ModuleInfo) -> Result<(), InstallError> {
	let path = content::get_module_deployment_path(options, &module.unique_id);
	if !path.exists() {
		return Err(InstallError::MissingContent)
	}

	let install_instructions = get_install_instructions(module, path, instance.game_dir()).unwrap();

	for (source, destination) in install_instructions {
		/* TODO: Fallback InstallMethods */
		let tracked = instance.tracked.get_file(&destination.to_string_lossy());
		if let Some(tracked) = tracked {
			match tracked.get_install_method() {
				crate::game_instance::filetracker::InstallMethod::Default => hardlink(source, destination)?,
				crate::game_instance::filetracker::InstallMethod::HardLink => hardlink(source, destination)?,
				crate::game_instance::filetracker::InstallMethod::Copy => copy(source, destination)?,
				crate::game_instance::filetracker::InstallMethod::Block => continue,
			}
		} else {
			hardlink(&source, &destination)?;
			instance.tracked.add_file(destination.to_string_lossy().to_string(), crate::game_instance::filetracker::InstallMethod::Default)
		}
	}

	Ok(())
}

/// Deciphers the install directives into a simpler (source, destination) tuple.
fn get_install_instructions(module: &ModuleInfo, extracted_archive: impl AsRef<Path>, game_dir: impl AsRef<Path>) -> Result<Vec<(PathBuf, PathBuf)>, std::io::Error> {
	let extracted_archive = extracted_archive.as_ref();
	let game_dir = game_dir.as_ref();

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
			game_dir.join(PathBuf::from(&directive.install_to))
			/* TODO: Check if the path exists, is valid, traversal attempts */
		};

		match &directive.source {
			SourceDirective::File(s) => {
				instruction.0 = extracted_archive.join(PathBuf::from(&s));
				instruction.1 = game_dir.join(instruction.1.join(PathBuf::from(&s).file_name().expect("weird paths in source directive")));
			},
			SourceDirective::Find(s) => {
				let mut queue = VecDeque::<PathBuf>::new();
				queue.push_back(extracted_archive.to_path_buf());
				'search: while let Some(p) = queue.pop_front() {
					for entry in p.read_dir()? {
						let entry = entry?;
						if entry.file_name().to_str().expect("filename isn't unicode") == s {
							instruction.0 = entry.path();
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