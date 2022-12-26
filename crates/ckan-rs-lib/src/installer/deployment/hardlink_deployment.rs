//! An installer which uses hard links to install mods without the need for copies making it much faster and more flexible.

use crate::installer::content;

pub struct HardLinkInstaller;

impl super::DeploymentMethod for HardLinkInstaller {
	fn deploy_content(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), super::DeploymentError> {
		let mut content = content::get_module_content(options, module)?;
		content.copy_to(options.deployment_dir())?;

		todo!()
	}

	fn remove_content(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), super::DeploymentError> {
		todo!()
	}
}