//! 

#[derive(Debug)]
pub enum ContentError {
	RequiredFieldMissing,
	ContentNotFound,
	/// The content type of the module is not currently supported.
	/// currently only zip is supported.
	UnsupportedContentType,
	IO(std::io::Error),
	Zip(zip::result::ZipError),
	FailedToExtract(zip::result::ZipError),
}

crate::error_wrapper!(ContentError, ContentError::IO, std::io::Error);
crate::error_wrapper!(ContentError, ContentError::Zip, zip::result::ZipError);

pub fn get_module_deployment_path(options: &crate::CkanRsOptions, id: &crate::metadb::ckan::ModUniqueIdentifier) -> std::path::PathBuf {
	options.deployment_dir().join(id.identifier.clone() + &id.version.to_string())
}

pub fn extract_content_to_deployment(options: &crate::CkanRsOptions, module: &crate::metadb::ModuleInfo) -> Result<(), ContentError> {
	if let Some(ct) = &module.download_content_type {
		if ct == "application/zip" {
			let download_path = super::download::get_module_download_path(options, &module.unique_id);
			let deploy_path = get_module_deployment_path(options, &module.unique_id);
			let mut zip = std::fs::File::open(download_path)
				.map_err(ContentError::IO)
				.and_then(|f|
					zip::ZipArchive::new(f).map_err(ContentError::Zip)
				)?;
			
			match zip.extract(deploy_path) {
				Ok(_) => Ok(()),
				Err(_) => todo!(), /* TODO: Clear left over files and return error */
			}
		} else {
			Err(ContentError::UnsupportedContentType)
		}
	} else {
		Err(ContentError::RequiredFieldMissing)
	}
}