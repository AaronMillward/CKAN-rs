//! Deployment is the a phase in the installer where we take a module and actually add it to the game.
//! 
//! This is done through an interface `ModuleDeployment`.

pub trait ModuleDeployment {
	fn deploy_module(game_dir: &std::path::Path, module: crate::metadb::ModuleInfo) -> crate::Result<()>;
	fn retract_module(game_dir: &std::path::Path, module: crate::metadb::ModuleInfo) -> crate::Result<()>;
}

mod hardlink_deployment;

/* TODO: Traditional copy based deployment */