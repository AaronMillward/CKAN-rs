//! Downloads a packages content.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
	/// Given package cannot be downloaded as it has no download information.
	#[error("given package does not have downloadable content.")]
	PackageMissingDownloadFields,
	#[error("reqwest error: {0}")]
	Reqwest(#[from] reqwest::Error),
	#[error("IO error: {0}")]
	IO(#[from] std::io::Error),
}

pub fn get_package_download_path(config: &crate::CkanRsConfig, id: &crate::metadb::package::PackageIdentifier) -> std::path::PathBuf {
	config.download_dir().join(id.identifier.clone() + &id.version.to_string() + ".zip")
}

/// Downloads multiple package's contents.
/// 
/// # Arguments
/// - `config` - Required for getting download paths.
/// - `client` - Client to download package contents with.
/// - `packages` - List of packages to download.
/// - `force` - Overwrite existing downloads.
pub async fn download_packages_content<'info>(config: &crate::CkanRsConfig, client: &reqwest::Client, packages: &[&'info crate::metadb::Package], force: bool) 
-> crate::Result<Vec<(&'info crate::metadb::Package, Result<std::path::PathBuf, DownloadError>)>> {
	let mut results = Vec::<(&crate::metadb::Package, Result<std::path::PathBuf, DownloadError>)>::new();
	
	for package in packages {
		let download_path = get_package_download_path(config, &package.identifier);
		if download_path.exists() && !force {
			log::info!("Package {} contents already downloaded, skipping.", &package.identifier);
			results.push((package, Ok(download_path)));
			continue;
		}
		
		let url = if let Some(url) = &package.download {
			url
		} else {
			results.push((package, Err(DownloadError::PackageMissingDownloadFields)));
			continue;
		};
		
		tokio::fs::create_dir_all(download_path.with_file_name("")).await?;
		let mut download_file = tokio::fs::File::create(&download_path).await?;
	
		log::info!("Downloading package {} from {}", package.identifier, url);
		let content = client
			.get(url)
			.send()
			.await?
			.bytes()
			.await?
			.to_vec();
	
		log::info!("Writing package download to disk: {}", package.identifier);
		tokio::io::copy(&mut content.as_slice(), &mut download_file).await?;
		
		/* TODO: Check SHA sums */
	
		results.push((package, Ok(download_path)));
	}

	Ok(results)
}