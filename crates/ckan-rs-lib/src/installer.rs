//! Handles installing modules to a game directory

/* 
I quickly searched through the metadb using vim and it seems like the *vast* majority of content types are `application/zip`
so we're just going to consider them all zips and error otherwise for now.
 */

use crate::ModuleInfo;

pub mod retrieval;
pub mod deployment;

/* TODO: `to_install` should be a two-dimensional array to handle install order */
pub async fn install(options: &crate::CkanRsOptions, to_install: &[ModuleInfo], game_dir: &std::path::Path) -> crate::Result<()> {
	let client = reqwest::Client::builder()
		.https_only(options.https_only())
		.build()?;
	
	for module in to_install {
		let mod_archive_path = retrieval::download_module_content(options.cache_dir(), &client, module).await?;
		// deployment::ModuleDeployment::deploy_module(options, game_dir, module);
	}

	todo!()
}

pub fn uninstall() {

}