//! Various settings used by many functions in this library.

use std::io::{Read, Write};

/// Config struct often passed into many functions.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CkanRsConfig {
	download_dir: std::path::PathBuf,
	data_dir: std::path::PathBuf,
	https_only: bool,
	do_checksums: bool,
}

impl Default for CkanRsConfig {
	fn default() -> Self {
		Self {
			download_dir: {
				#[cfg(target_os = "windows")]
				let path = std::path::PathBuf::from(std::env::var("APPDATA").expect("APPDATA misssing."));

				#[cfg(not(target_os = "windows"))]
				let path = if let Ok(e) = std::env::var("XDG_CACHE_HOME") {
					std::path::PathBuf::from(e) 
				} else {
					std::path::PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set.")).join(".cache")
				};

				let path = path.join("CKAN-rs").join("downloads");
				std::fs::create_dir_all(&path).expect("failed to create downloads directory.");
				path
			},
			data_dir: {
				#[cfg(target_os = "windows")]
				let path = std::path::PathBuf::from(std::env::var("APPDATA").expect("APPDATA misssing."));

				#[cfg(not(target_os = "windows"))]
				let path = if let Ok(e) = std::env::var("XDG_DATA_HOME") {
					std::path::PathBuf::from(e) 
				} else {
					std::path::PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set.")).join(".local/share")
				};

				let path = path.join("CKAN-rs").join("data");
				std::fs::create_dir_all(&path).expect("failed to create data directory.");
				path
			},
			https_only: true,
			do_checksums: true,
		}
	}
}

impl CkanRsConfig {
	pub fn download_dir(&self) -> &std::path::PathBuf {
		&self.download_dir
	}
	/// returns if the directory is valid and was set or not.
	pub fn set_download_dir(&mut self, download_dir: std::path::PathBuf) -> bool {
		if download_dir.is_dir() {
			self.download_dir = download_dir;
			true
		} else {
			false
		}
	}

	pub fn data_dir(&self) -> &std::path::PathBuf {
		&self.data_dir
	}
	/// returns if the directory is valid and was set or not.
	pub fn set_data_dir(&mut self, data_dir: std::path::PathBuf) -> bool {
		if data_dir.is_dir() {
			self.data_dir = data_dir;
			true
		} else {
			false
		}
	}

	pub fn https_only(&self) -> bool {
		self.https_only
	}
	pub fn set_https_only(&mut self, https_only: bool) {
		self.https_only = https_only;
	}

	pub fn get_do_checksums(&self) -> bool {
		self.do_checksums
	}
	pub fn set_do_checksums(&mut self, do_checksums: bool) {
		self.do_checksums = do_checksums;
	}

	/// Loads the config file from a file.
	/// 
	/// # Platform Specific
	/// On Windows this file is located at `%appdata%/CKAN-rs/config.json`
	/// 
	/// On other platforms it is located at `$XDG_CONFIG_HOME/CKAN-rs/config.json`
	/// 
	/// # Errors
	/// - [`IO`](crate::error::Error::IO) when opening or reading from the file.
	/// - [`SerdeJSON`](crate::error::Error::SerdeJSON) when deserializing the file.
	pub fn load_from_disk() -> crate::Result<Self> {
		let path = get_config_path();

		let mut file = std::fs::File::open(path)?;
		let mut s: String = Default::default();
		file.read_to_string(&mut s)?;
		Ok(serde_json::from_str(&s)?)
	}

	/// Saves the config to a JSON file.
	/// 
	/// # Platform Specific
	/// On Windows this file is located at `%appdata%/CKAN-rs/config.json`
	/// 
	/// On other platforms it is located at `$XDG_CONFIG_HOME/CKAN-rs/config.json`
	/// 
	/// # Errors
	/// - [`IO`](crate::error::Error::IO) when opening the file, writing to it or creating it's parent directories.
	/// - [`SerdeJSON`](crate::error::Error::SerdeJSON) when serializing the file.
	pub fn save_to_disk(&self) -> crate::Result<()> {
		let path = get_config_path();
		std::fs::create_dir_all(path.with_file_name(""))?;
		let json = serde_json::to_string_pretty(self)?;
		let mut file = std::fs::File::create(path)?;
		file.write_all(json.as_bytes())?;
		Ok(())
	}
}

fn get_config_path() -> std::path::PathBuf {
	#[cfg(target_os = "windows")]
	let path = std::path::PathBuf::from(std::env::var("APPDATA").expect("APPDATA misssing."));

	#[cfg(not(target_os = "windows"))]
	let path = if let Ok(e) = std::env::var("XDG_CONFIG_HOME") {
		std::path::PathBuf::from(e) 
	} else {
		std::path::PathBuf::from(std::env::var("HOME").expect("HOME environment variable not set.")).join(".config")
	};

	path.join("CKAN-rs").join("config.json")
}