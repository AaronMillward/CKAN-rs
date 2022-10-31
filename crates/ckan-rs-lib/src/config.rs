pub struct CkanRsOptions {
	cache_dir: std::path::PathBuf,
	https_only: bool,
}

impl CkanRsOptions {
	pub fn cache_dir(&self) -> &std::path::PathBuf {
		&self.cache_dir
	}
	/// returns if the directory is valid or not.
	pub fn set_cache_dir(&mut self, cache_dir: std::path::PathBuf) -> bool {
		if cache_dir.is_dir() {
			self.cache_dir = cache_dir;
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