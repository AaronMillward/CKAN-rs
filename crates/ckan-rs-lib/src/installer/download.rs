//! Downloads a packages content.

#[derive(Debug)]
pub enum DownloadError {
	/// Given package cannot be downloaded as it has no download information.
	PackageMissingDownloadFields,
	/// There's no need to download this package it's already in the downloads cache.
	ContentAlreadyDownloaded,
	Reqwest(reqwest::Error),
	IO(std::io::Error),
}

crate::error_wrapper!(DownloadError, DownloadError::Reqwest, reqwest::Error);
crate::error_wrapper!(DownloadError, DownloadError::IO     , std::io::Error);

pub fn get_package_download_path(options: &crate::CkanRsOptions, id: &crate::metadb::ckan::PackageIdentifier) -> std::path::PathBuf {
	options.download_dir().join(id.identifier.clone() + &id.version.to_string())
}

// Downloads multiple packages contents.
pub async fn download_packages_content<'info>(options: &crate::CkanRsOptions, client: &reqwest::Client, packages: &[&'info crate::metadb::Package]) -> Vec<(&'info crate::metadb::Package, Result<std::path::PathBuf, DownloadError>)> {
	let download_directory = options.download_dir();
	
	let mut results = Vec::<(&crate::metadb::Package, Result<std::path::PathBuf, DownloadError>)>::new();
	
	for package in packages {
		/* TODO: unwraps */

		let download_path = download_directory.join(package.identifier.identifier.clone() + &package.identifier.version.to_string());
		if !download_path.exists() {
			results.push((package, Err(DownloadError::ContentAlreadyDownloaded)));
			continue;
		}
		
		let url = if let Some(url) = &package.download {
			url
		} else {
			results.push((package, Err(DownloadError::PackageMissingDownloadFields)));
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
	
		eprintln!("Writing package download to disk: {}", package.identifier);
		tokio::io::copy(&mut content.as_slice(), &mut download_file).await.unwrap();
		
		/* TODO: Check SHA sums */
	
		results.push((package, Ok(download_path)));
	}

	/* TODO: Save metadb with new download paths */

	results
}