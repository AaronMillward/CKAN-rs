use std::path::Path;

#[derive(Debug)]
pub enum DeploymentError {
	MissingContent,
	HardLink(std::io::Error),
	Copy(std::io::Error),
}

fn hardlink(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<(), DeploymentError> {
	std::fs::hard_link(source, destination).map_err(DeploymentError::HardLink)
}

fn copy(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> Result<(), DeploymentError> {
	std::fs::copy(source, destination).map(|_|()).map_err(DeploymentError::Copy)
}

pub async fn deploy_module(options: &crate::CkanRsOptions, instance: &mut crate::game_instance::GameInstance, module: &crate::ModuleInfo) -> Result<(), DeploymentError> {
	let path = super::content::get_module_deployment_path(options, &module.unique_id);
	if !path.exists() {
		return Err(DeploymentError::MissingContent)
	}

	let install_instructions = super::get_install_instructions(module, path, instance.game_dir()).unwrap();

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