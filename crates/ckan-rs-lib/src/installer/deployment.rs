//! Deployment is the a phase in the installer where we take a module and actually add it to the game.
//! 
//! This is done through an interface `ModuleDeployment`.

#[derive(Debug)]
pub enum DeploymentError {
	UndeployableModule,
	ModuleArchiveDoesNotExist,
	ContentNotFound,
	IO(std::io::Error),
	Zip(zip::result::ZipError),
	FailedToExtract(zip::result::ZipError),
}

crate::error_wrapper!(DeploymentError, DeploymentError::IO, std::io::Error);
crate::error_wrapper!(DeploymentError, DeploymentError::Zip, zip::result::ZipError);

#[derive(Debug)]
pub enum DeploymentMethod {
	HardLink,
}

pub trait ModuleDeployment {
	fn deploy_module(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), DeploymentError>;
	fn retract_module(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), DeploymentError>;
}

pub mod hardlink_deployment;

/* TODO: Traditional copy based deployment */

/// 
pub fn get_or_extract_mod_archive_for_deployment(options: &crate::CkanRsOptions, module: &crate::metadb::ckan::ModUniqueIdentifier) -> Result<std::path::PathBuf, DeploymentError>{
	let archive_extracted_dir = options.deployment_dir().join(module.to_string());
	if archive_extracted_dir.exists() {
		return Ok(archive_extracted_dir)
	}

	let archive_source = options.download_dir().join(module.to_string());
	if !archive_source.exists() {
		return Err(DeploymentError::ModuleArchiveDoesNotExist)
	}

	/* Extract from archive */ {
		let f = std::fs::File::open(archive_source)?;
		let archive = zip::ZipArchive::new(f)?;
		archive.extract(archive_extracted_dir).map_err(|e| DeploymentError::FailedToExtract(e))?;
		/* 
		TODO:
		XXX:
		According to zip an error from extract can leave the directory in an invalid state
		I don't really want to "sudo rm -rf /" without the user so this error can ride all the way up.
		 */
	}

	return Ok(archive_extracted_dir);
}