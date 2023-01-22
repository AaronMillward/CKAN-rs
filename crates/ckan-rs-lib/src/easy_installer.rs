//! Easy Installer
//! 
//! Merges the download, extract and install steps into a single function [`install`] for convienience.

use crate::installer::*;
use crate::metadb::ckan::ModUniqueIdentifier as ModID;
use crate::ModuleInfo;
use crate::game_instance::GameInstance;

#[derive(Debug)]
pub enum EasyInstallerError<'id> {
	DownloadsFailed(Vec<(&'id ModID, download::DownloadError)>),
	ExtractFailed(Vec<(&'id ModID, content::ContentError)>),
	Reqwest(reqwest::Error),
}

crate::error_wrapper!(EasyInstallerError<'_>, EasyInstallerError::Reqwest, reqwest::Error);

/* TODO: `to_install` should be a two-dimensional array to handle install order */
pub async fn install<'id>(options: &crate::CkanRsOptions, instance: &mut GameInstance, to_install: &'id [ModuleInfo]) -> Result<(), EasyInstallerError<'id>> {
	download_modules(options, to_install).await?;
	extract_content(options, to_install).await?;

	/* Install Content */
	for module in to_install {
		install_module(options, instance, module).await.expect("install failed.");
	}

	Ok(())
}

async fn download_modules<'id>(options: &crate::CkanRsOptions, to_install: &'id [ModuleInfo]) -> Result<(), EasyInstallerError<'id>> {
	let client = reqwest::Client::builder()
		.https_only(options.https_only())
		.build()?;

	let result = download::download_modules_content(options.download_dir(), &client, to_install).await;
	let result_failed = result
		.into_iter()
		.filter(|e| e.1.is_err())
		.map(|e| (&e.0.unique_id, e.1.unwrap_err()))
		.collect::<Vec<_>>();
	
	if result_failed.is_empty() {
		Ok(())
	}
	else {
		Err(EasyInstallerError::DownloadsFailed(result_failed))
	}
}

async fn extract_content<'id>(options: &crate::CkanRsOptions, to_install: &'id [ModuleInfo]) -> Result<(), EasyInstallerError<'id>> {
	let mut failed = Vec::<(&ModID, content::ContentError)>::new();
	
	for module in to_install {
		let result = content::extract_content_to_deployment(options, module);
		match result {
			Ok(_) => (),
			Err(e) => failed.push((&module.unique_id, e)),
		}
	}

	if failed.is_empty() {
		Ok(())
	} else {
		Err(EasyInstallerError::ExtractFailed(failed))
	}
}