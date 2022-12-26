//! Acquirement is when we take the module and download it's contents

#[derive(Debug)]
pub enum RetrievalError {
	/// Given module cannot be downloaded as it has no download information.
	ModuleMissingDownloadFields,
	/// There's no need to download this module it's already in the downloads cache.
	ContentAlreadyDownloaded,
	Reqwest(reqwest::Error),
	IO(std::io::Error),
}

crate::error_wrapper!(RetrievalError, RetrievalError::Reqwest, reqwest::Error);
crate::error_wrapper!(RetrievalError, RetrievalError::IO     , std::io::Error);

// Downloads a modules content.
pub async fn download_module_content(download_directory: &std::path::Path, client: &reqwest::Client, module: &crate::metadb::ModuleInfo) -> Result<std::path::PathBuf, RetrievalError> {
	let download_path = download_directory.join(module.unique_id.identifier.clone() + &module.unique_id.version.to_string());
	if !download_path.exists() {
		return Err(RetrievalError::ContentAlreadyDownloaded)
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