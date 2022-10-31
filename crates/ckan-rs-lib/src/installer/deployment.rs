//! Deployment is the a phase in the installer where we take a module and actually add it to the game.
//! 
//! This is done through an interface `ModuleDeployment`.

pub enum DeploymentError {
	UndeployableModule,

}

pub enum DeploymentMethod {
	HardLink,
}

pub trait ModuleDeployment {
	fn deploy_module(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), DeploymentError>;
	fn retract_module(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), DeploymentError>;
}

mod hardlink_deployment;

/* TODO: Traditional copy based deployment */