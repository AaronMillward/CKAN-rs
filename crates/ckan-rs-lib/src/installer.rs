//! Handles installing modules to a game directory

/* 
I quickly searched through the metadb using vim and it seems like the *vast* majority of content types are `application/zip`
so we're just going to consider them all zips and error otherwise for now.
 */

use std::collections::VecDeque;
use std::path::PathBuf;
use std::path::Path;

use crate::ModuleInfo;
use crate::metadb::ckan::InstallDirective;
use crate::metadb::ckan::SourceDirective;

pub mod retrieval;
pub mod content;
pub mod deployment;
use deployment::DeploymentMethod;

/* TODO: `to_install` should be a two-dimensional array to handle install order */
pub async fn install<D: DeploymentMethod>(options: &crate::CkanRsOptions, to_install: &[ModuleInfo], game_dir: &std::path::Path) -> crate::Result<()> {
	let client = reqwest::Client::builder()
		.https_only(options.https_only())
		.build()?;
	
	for module in to_install {
		/* TODO: to_install to also list the modules deployment method to allow per module deployment settings */
		
		retrieval::download_module_content(options.download_dir(), &client, module).await?;
		
		// let install_instructions = get_install_instructions(module, deployment_path, game_dir)?;

		D::deploy_content(options, game_dir, module);
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