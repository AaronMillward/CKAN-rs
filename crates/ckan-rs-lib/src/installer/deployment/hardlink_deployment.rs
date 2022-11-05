//! An installer which uses hard links to install mods without the need for copies making it much faster and more flexible.

use crate::metadb::ckan::{SourceDirective, InstallDirective, OptionalDirective};

pub struct HardLinkInstaller;

impl super::ModuleDeployment for HardLinkInstaller {
	
	fn deploy_module(options: &crate::CkanRsOptions, game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), super::DeploymentError> {
		if module.install.is_empty() {
			/* If no install sections are provided, a CKAN client must find the top-most directory in the archive that matches the module identifier, and install that with a target of GameData. In other words, the default install section is: */
			todo!()
		}
		let mut file_list = Vec::<String>::new();
		for directive in module.install {
			let source = match directive.source {
				
				SourceDirective::File(s) =>  s /* TODO: MyMods/KSP/Foo will be installed into GameData/Foo */,
				SourceDirective::Find(s) => {

				},
				SourceDirective::FindRegExp(s) => todo!() /* TODO: Regex */,
			}
		}

		todo!()
	}

	fn retract_module(game_dir: &std::path::Path, module: &crate::metadb::ModuleInfo) -> Result<(), super::DeploymentError> {
		todo!()
	}
}