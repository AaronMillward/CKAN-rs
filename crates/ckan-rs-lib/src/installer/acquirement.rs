//! Acquirement is when we take the module and download it's contents

pub enum AcquirementError {
	/// Given module cannot be downloaded as it has no download information.
	ModuleMissingDownloadFields,
	ReqwestError(reqwest::Error),
	IOError(std::io::Error),
}

crate::error_wrapper!(AcquirementError, AcquirementError::ReqwestError, reqwest::Error);
crate::error_wrapper!(AcquirementError, AcquirementError::IOError     , std::io::Error);

pub async fn download_module(cache_dir: &std::path::Path, client: &reqwest::Client, module: &crate::metadb::ModuleInfo) -> Result<std::path::PathBuf, AcquirementError> {
	let download_path = cache_dir.with_file_name(module.unique_id.identifier.clone() + &module.unique_id.version.to_string());
	if download_path.exists() {
		eprintln!("Module download already exists! {}", module.unique_id);
		return Ok(download_path);
	}

	let url = if let Some(url) = &module.download {
		url
	} else {
		return Err(AcquirementError::ModuleMissingDownloadFields)
	};

	let b = client
		.get(url)
		.send()
		.await?
		.bytes()
		.await?
		.to_vec();
		
	/* TODO: Check SHA sums */

	eprintln!("Writing module download to disk: {}", module.unique_id);
	tokio::fs::create_dir_all(&download_path).await?;
	tokio::fs::write(&download_path, &b).await?;

	Ok(download_path)
}