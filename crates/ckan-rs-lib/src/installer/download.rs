//! Downloads a packages content.

#[derive(Debug)]
pub enum DownloadError {
	/// Given package cannot be downloaded as it has no download information.
	PackageMissingDownloadFields,
	Reqwest(reqwest::Error),
	IO(std::io::Error),
}

crate::error_wrapper!(DownloadError, DownloadError::Reqwest, reqwest::Error);
crate::error_wrapper!(DownloadError, DownloadError::IO     , std::io::Error);

pub fn get_package_download_path(options: &crate::CkanRsOptions, id: &crate::metadb::ckan::PackageIdentifier) -> std::path::PathBuf {
	options.download_dir().join(id.identifier.clone() + &id.version.to_string() + ".zip")
}

/// Downloads multiple package's contents.
/// 
/// # Arguments
/// - `options` - Required for getting download paths.
/// - `client` - Client to download package contents with.
/// - `packages` - List of packages to download.
/// - `force` - Overwrite existing downloads.
pub async fn download_packages_content<'info>(options: &crate::CkanRsOptions, client: &reqwest::Client, packages: &[&'info crate::metadb::Package], force: bool) -> Vec<(&'info crate::metadb::Package, Result<std::path::PathBuf, DownloadError>)> {
	let mut results = Vec::<(&crate::metadb::Package, Result<std::path::PathBuf, DownloadError>)>::new();
	
	for package in packages {
		/* TODO: unwraps */

		let download_path = get_package_download_path(options, &package.identifier);
		if download_path.exists() && !force {
			log::debug!("Package contents already downloaded, skipping.");
			results.push((package, Ok(download_path)));
			continue;
		}
		
		let url = if let Some(url) = &package.download {
			url
		} else {
			results.push((package, Err(DownloadError::PackageMissingDownloadFields)));
			continue;
		};
		
		tokio::fs::create_dir_all(download_path.with_file_name("")).await.unwrap();
		let mut download_file = tokio::fs::File::create(&download_path).await.unwrap();
	
		log::info!("Downloading package {} from {}", package.identifier, url);
		let content = client
			.get(url)
			.send()
			.await.unwrap()
			.bytes()
			.await.unwrap()
			.to_vec();
	
		log::info!("Writing package download to disk: {}", package.identifier);
		tokio::io::copy(&mut content.as_slice(), &mut download_file).await.unwrap();
		
		/* TODO: Check SHA sums */
	
		results.push((package, Ok(download_path)));
	}

	/* TODO: Save metadb with new download paths */

	results
}