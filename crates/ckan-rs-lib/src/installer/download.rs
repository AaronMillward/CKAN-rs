//! Downloads a modules content.

#[derive(Debug)]
pub enum DownloadError {
	/// Given module cannot be downloaded as it has no download information.
	ModuleMissingDownloadFields,
	/// There's no need to download this module it's already in the downloads cache.
	ContentAlreadyDownloaded,
	Reqwest(reqwest::Error),
	IO(std::io::Error),
}

crate::error_wrapper!(DownloadError, DownloadError::Reqwest, reqwest::Error);
crate::error_wrapper!(DownloadError, DownloadError::IO     , std::io::Error);

pub fn get_module_download_path(options: &crate::CkanRsOptions, id: &crate::metadb::ckan::ModUniqueIdentifier) -> std::path::PathBuf {
	options.download_dir().join(id.identifier.clone() + &id.version.to_string())
}

// Downloads multiple modules contents.
pub async fn download_modules_content<'info>(options: &crate::CkanRsOptions, client: &reqwest::Client, modules: &[&'info crate::metadb::ModuleInfo]) -> Vec<(&'info crate::metadb::ModuleInfo, Result<std::path::PathBuf, DownloadError>)> {
	let download_directory = options.download_dir();
	
	let mut results = Vec::<(&crate::metadb::ModuleInfo, Result<std::path::PathBuf, DownloadError>)>::new();
	
	for module in modules {
		/* TODO: unwraps */

		let download_path = download_directory.join(module.unique_id.identifier.clone() + &module.unique_id.version.to_string());
		if !download_path.exists() {
			results.push((module, Err(DownloadError::ContentAlreadyDownloaded)));
			continue;
		}
		
		let url = if let Some(url) = &module.download {
			url
		} else {
			results.push((module, Err(DownloadError::ModuleMissingDownloadFields)));
			continue;
		};
		
		let mut download_file = tokio::fs::File::create(&download_path).await.unwrap();
	
		let content = client
			.get(url)
			.send()
			.await.unwrap()
			.bytes()
			.await.unwrap()
			.to_vec();
	
		eprintln!("Writing module download to disk: {}", module.unique_id);
		tokio::io::copy(&mut content.as_slice(), &mut download_file).await.unwrap();
		
		/* TODO: Check SHA sums */
	
		results.push((module, Ok(download_path)));
	}

	/* TODO: Save metadb with new download paths */

	results
}