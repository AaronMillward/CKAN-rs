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

pub trait Content {
	fn copy_to(&mut self, dir: &std::path::Path) -> Result<(), ContentError>;
}

pub fn get_module_content(options: &crate::CkanRsOptions, module: &crate::metadb::ModuleInfo) -> Result<Box<dyn Content>, ContentError> {
	if let Some(ct) = &module.download_content_type {
		let download_path = options.download_dir().join(module.unique_id.identifier.clone() + &module.unique_id.version.to_string());
		if ct == "application/zip" {
			return Ok(Box::new(ZipContent::new(&download_path)?))
		} else {
			return Err(ContentError::UnsupportedContentType);
		}
	} else {
		return Err(ContentError::RequiredFieldMissing)
	}
}

pub struct ZipContent {
	archive: zip::ZipArchive<std::fs::File>,
}

impl ZipContent {
	pub fn new(content: &std::path::Path) -> Result<ZipContent, ContentError> {
		let f = std::fs::File::open(content)?;
		let archive = zip::ZipArchive::new(f)?;
		Ok(ZipContent { archive })
	}
}

impl Content for ZipContent {
	fn copy_to(&mut self, dir: &std::path::Path) -> Result<(), ContentError>{
		self.archive.extract(dir).map_err(ContentError::FailedToExtract)?;
		/* 
			TODO:
			XXX:
			According to zip an error from extract can leave the directory in an invalid state
			I don't really want to "sudo rm -rf /" without the user so this error can ride all the way up.
		*/
		Ok(())
	}
}