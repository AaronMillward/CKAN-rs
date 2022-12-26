//! Deployment is the a phase in the installer where we take a module and actually add it to the game.
//! 
//! This is done through an interface `ModuleDeployment`.

#[derive(Debug)]
pub enum DeploymentError {
	UndeployableModule,
	ModuleArchiveDoesNotExist,
	ContentError(super::content::ContentError),
	IO(std::io::Error),
}

crate::error_wrapper!(DeploymentError, DeploymentError::IO, std::io::Error);
crate::error_wrapper!(DeploymentError, DeploymentError::ContentError, super::content::ContentError);

pub trait DeploymentMethod {
	fn deploy_content(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), DeploymentError>;
	fn remove_content(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), DeploymentError>;
}

pub mod hardlink_deployment;

/* TODO: Traditional copy based deployment */