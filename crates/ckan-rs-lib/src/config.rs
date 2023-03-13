pub struct CkanRsOptions {
	download_dir: std::path::PathBuf,
	data_dir: std::path::PathBuf,
	https_only: bool,
}

impl Default for CkanRsOptions {
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
			https_only: true
		}
	}
}

impl CkanRsOptions {
	pub fn download_dir(&self) -> &std::path::PathBuf {
		&self.download_dir
	}
	/// returns if the directory is valid or not.
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
	/// returns if the directory is valid or not.
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
}