//! Downloads a packages content.

use thiserror::Error;
use crate::metadb::package::*;

/// Errors that can occur during the download process.
#[derive(Debug, Error)]
pub enum DownloadError {
	/// Given package cannot be downloaded as it has no download information.
	#[error("given package does not have downloadable content.")]
	PackageMissingDownloadFields,
	/// The downloaded content hash does not match hash in the package.
	#[error("downloaded content hash does not match hash in package.")]
	DifferentHashes,
	#[error("reqwest error: {0}")]
	Reqwest(#[from] reqwest::Error),
	#[error("IO error: {0}")]
	IO(#[from] std::io::Error),
}

pub fn get_package_download_path(config: &crate::Config, id: &crate::metadb::package::PackageIdentifier) -> std::path::PathBuf {
	config.download_dir().join(id.identifier.clone() + &id.version.to_string() + ".zip")
}

/// Downloads multiple package's contents.
/// 
/// # Parameters
/// - `config` - Required for getting download paths.
/// - `packages` - List of packages to download.
/// - `force` - Overwrite existing downloads.
/// 
/// # Returns
/// A vector of tuples containing a package to be downloaded and a result of the download.
pub async fn download_packages_content<'info>(config: &crate::Config, packages: &[&'info Package], force: bool) 
-> Vec<(&'info Package, Result<std::path::PathBuf, DownloadError>)> {

	async fn download_package(config: &crate::Config, client: &reqwest::Client, package: &Package, force: bool)
	-> Result<std::path::PathBuf, DownloadError> {
		let download_path = get_package_download_path(config, &package.identifier);
		if download_path.exists() && !force {
			log::info!("Package {} contents already downloaded, skipping.", &package.identifier);
			return Ok(download_path);
		}
		
		let url = if let Some(url) = &package.download {
			url
		} else {
			return Err(DownloadError::PackageMissingDownloadFields);
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
		
		if config.get_do_checksums() {
			if let Some(package_hash) = &package.download_hash_sha256 {
				let content_hash = sha256::digest(content.as_slice());
				if String::from_utf8_lossy(package_hash) != content_hash {
					return Err(DownloadError::DifferentHashes);
				}
			}
			else if let Some(_package_hash) = &package.download_hash_sha1 {
				/* TODO: Sha1 hashing */
				/* XXX: Sha1 crate doesn't work as the documentation for it says so we need to find a better crate for this */
				unimplemented!("Sha1 hashing not yet implemented.")
			}
		}

		Ok(download_path)
	}

	let mut results = Vec::<(&Package, Result<std::path::PathBuf, DownloadError>)>::new();

	let client = reqwest::Client::builder()
		.https_only(config.https_only())
		.build()
		.expect("failed to create reqwest client.");

	for package in packages {
		results.push(
			(
				package, 
				download_package(config, &client, package, force).await,
			)
		);
	}

	results
}