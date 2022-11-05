//! Acquirement is when we take the module and download it's contents

#[derive(Debug)]
pub enum RetrievalError {
	/// Given module cannot be downloaded as it has no download information.
	ModuleMissingDownloadFields,
	/// The module's content has not been downloaded yet.
	ContentNotCached,
	/// The content type of the module is not currently supported.
	/// currently only zip is supported.
	UnsupportedContentType,
	Reqwest(reqwest::Error),
	IO(std::io::Error),
}

crate::error_wrapper!(RetrievalError, RetrievalError::Reqwest, reqwest::Error);
crate::error_wrapper!(RetrievalError, RetrievalError::IO     , std::io::Error);

/// Gets a modules content only by retrieving it from the cache.
pub fn get_module_content(cache_dir: &std::path::Path, module: &crate::metadb::ckan::ModUniqueIdentifier) -> Result<std::path::PathBuf, RetrievalError> {
	let download_path = cache_dir.with_file_name(module.identifier.clone() + &module.version.to_string());
	if download_path.exists() {
		Ok(download_path)
	} else {
		Err(RetrievalError::ContentNotCached)
	}
}

/// Gets a modules content either by retrieving it from the cache or downloading it.
pub async fn download_or_get_module_content(download_directory: &std::path::Path, client: &reqwest::Client, module: &crate::metadb::ModuleInfo) -> Result<std::path::PathBuf, RetrievalError> {
	if let Some(ct) = &module.download_content_type {
		if ct != "application/zip" {
			return Err(RetrievalError::UnsupportedContentType);
		}
	} else {
		return Err(RetrievalError::ModuleMissingDownloadFields);
	}

	let download_path = download_directory.join(module.unique_id.identifier.clone() + &module.unique_id.version.to_string());
	if download_path.exists() {
		return Ok(download_path)
	}

	let url = if let Some(url) = &module.download {
		url
	} else {
		return Err(RetrievalError::ModuleMissingDownloadFields)
	};

	let mut download_file = tokio::fs::File::create(&download_path).await?;

	let content = client
		.get(url)
		.send()
		.await?
		.bytes()
		.await?
		.to_vec();

	eprintln!("Writing module download to disk: {}", module.unique_id);
	tokio::io::copy(&mut content.as_slice(), &mut download_file).await?;
	
	/* TODO: Check SHA sums */

	Ok(download_path)
}