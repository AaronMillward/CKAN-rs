//! Package downloadable content extraction.

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
	/// The package is not installable. likely due to [`kind`](crate::metadb::package::Package::kind) not being [`Package`](crate::metadb::package::Kind::Package)
	#[error("package is not installable.")]
	PackageNotInstallable,
	/// The package uses an unsupported content type.
	#[error("package uses an unsupported content type.")]
	UnsupportedContentType,
	#[error("IO error: {0}")]
	IO(#[from] std::io::Error),
	#[error("zip error: {0}")]
	Zip(#[from] zip::result::ZipError),
}

impl crate::game_instance::GameInstance {
	pub fn get_package_deployment_path(&self, id: impl AsRef<crate::metadb::package::PackageIdentifier>) -> std::path::PathBuf {
		let id = id.as_ref();
		self.deployment_dir.join(id.identifier.clone() + &id.version.to_string())
	}
}

/* TODO: Remove from public API */
/// Extracts a packages contents to the deployment directory of a given instance.
/// 
/// # Parameters
/// - `config` - Contains the download cache.
/// - `instance` - The game instance to extract the content for.
/// - `package` - The package to extract.
/// 
/// # Errors
/// - Returns [`ContentError::PackageNotInstallable`] when given a metapackage or dlc which have no installable content.
/// - Currently only zip is supported and so returns [`ContentError::UnsupportedContentType`] if any other content type is provided.
pub fn extract_content_to_deployment(config: &crate::Config, instance: &crate::game_instance::GameInstance, package: &crate::metadb::package::Package) -> Result<(), ContentError> {
	let ct = package.download_content_type.as_ref().ok_or(ContentError::PackageNotInstallable)?;
	if ct == "application/zip" {
		let download_path = super::download::get_package_download_path(config, &package.identifier);
		let deploy_path = instance.get_package_deployment_path(package);
		let mut zip = zip::ZipArchive::new(
			std::fs::File::open(download_path)?
		)?;
		
		std::fs::create_dir_all(deploy_path.with_file_name(""))?;
		match zip.extract(deploy_path) {
			Ok(_) => Ok(()),
			Err(_) => todo!(), /* TODO: Clear left over files and return error */
		}
	} else {
		Err(ContentError::UnsupportedContentType)
	}
}