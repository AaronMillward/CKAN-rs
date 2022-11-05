pub struct CkanRsOptions {
	download_dir: std::path::PathBuf,
	deployment_dir: std::path::PathBuf,
	https_only: bool,
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

	pub fn deployment_dir(&self) -> &std::path::PathBuf {
		&self.download_dir
	}
	/// returns if the directory is valid or not.
	pub fn set_deployment_dir(&mut self, deployment_dir: std::path::PathBuf) -> bool {
		if deployment_dir.is_dir() {
			self.deployment_dir = deployment_dir;
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