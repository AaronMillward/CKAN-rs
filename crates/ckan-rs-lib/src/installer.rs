//! Handles installing modules to a game directory

/* 
I quickly searched through the metadb using vim and it seems like the *vast* majority of content types are `application/zip`
so we're just going to consider them all zips and error otherwise for now.
 */

use std::path::PathBuf;

use crate::ModuleInfo;

pub mod retrieval;
pub mod deployment;
use deployment::ModuleDeployment;


/* TODO: `to_install` should be a two-dimensional array to handle install order */
pub async fn install(options: &crate::CkanRsOptions, to_install: &[ModuleInfo], game_dir: &std::path::Path) -> crate::Result<()> {
	let client = reqwest::Client::builder()
		.https_only(options.https_only())
		.build()?;
	
	for module in to_install {
		/* TODO: to_install to also list the modules deployment method to allow per module deployment settings */
		
		/* XXX: as of writing only returns zip files */
		let mod_archive_path = retrieval::download_or_get_module_content(options.download_dir(), &client, module).await?;
		
		let deployment_path = deployment::get_or_extract_mod_archive_for_deployment(options, &module.unique_id).unwrap();

		let install_instructions = Vec::<(PathBuf, PathBuf)>::new();
		/* determine files to link using install directives */ {
			if module.install.is_empty() {
				/* "If no install sections are provided, a CKAN client must find 
				the top-most directory in the archive that matches the module identifier,
				 and install that with a target of GameData." */
				todo!()
			}

			for directive in module.install {
				let mut instruction: (PathBuf, PathBuf);

				instruction.1 = if directive.install_to == "GameRoot" {
					todo!("GameRoot install directive not yet supported")
				} else {
					game_dir.join(directive.install_to.into())
				};

				let source = match directive.source {
					SourceDirective::File(s) => {
						instruction.0 = s;
						instruction.1
						todo!()
					},
					SourceDirective::Find(s) => {
						std::fs::read_dir(deployment_path)
					},
					SourceDirective::FindRegExp(s) => todo!() /* TODO: Regex */,
				}
			}
		}


		deployment::hardlink_deployment::HardLinkInstaller::deploy_module(options, game_dir, module);
	}

	todo!()
}

pub fn uninstall() {

}