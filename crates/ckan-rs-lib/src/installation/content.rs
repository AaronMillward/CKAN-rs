//! 

#[derive(Debug)]
pub enum ContentError {
	PackageNotInstallable,
	UnsupportedContentType,
	IO(std::io::Error),
	Zip(zip::result::ZipError),
}

crate::error_wrapper!(ContentError, ContentError::IO, std::io::Error);
crate::error_wrapper!(ContentError, ContentError::Zip, zip::result::ZipError);

impl crate::game_instance::GameInstance {
	pub fn get_package_deployment_path(&self, id: impl AsRef<crate::metadb::package::PackageIdentifier>) -> std::path::PathBuf {
		let id = id.as_ref();
		self.deployment_dir.join(id.identifier.clone() + &id.version.to_string())
	}
}

/* TODO: Remove from public API */
/// Extracts a packages contents to the deployment directory of a given instance.
/// 
/// # Arguments
/// - `config` - Contains the download cache.
/// - `instance` - The game instance to extract the content for.
/// - `package` - The package to extract.
/// 
/// # Errors
/// - Returns [`ContentError::PackageNotInstallable`] when given a metapackage or dlc which have no installable content.
/// - Currently only zip is supported and so returns [`ContentError::UnsupportedContentType`] if any other content type is provided.
pub fn extract_content_to_deployment(config: &crate::CkanRsConfig, instance: &crate::game_instance::GameInstance, package: &crate::metadb::Package) -> Result<(), ContentError> {
	let ct = package.download_content_type.as_ref().ok_or(ContentError::PackageNotInstallable)?;
	if ct == "application/zip" {
		let download_path = super::download::get_package_download_path(config, &package.identifier);
		let deploy_path = instance.get_package_deployment_path(package);
		let mut zip = std::fs::File::open(download_path)
			.map_err(ContentError::IO)
			.and_then(|f|
				zip::ZipArchive::new(f).map_err(ContentError::Zip)
			)?;
		
		std::fs::create_dir_all(deploy_path.with_file_name("")).unwrap();
		match zip.extract(deploy_path) {
			Ok(_) => Ok(()),
			Err(_) => todo!(), /* TODO: Clear left over files and return error */
		}
	} else {
		Err(ContentError::UnsupportedContentType)
	}
}