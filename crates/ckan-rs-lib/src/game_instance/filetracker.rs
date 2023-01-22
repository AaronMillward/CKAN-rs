use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum InstallMethod {
	Default,
	HardLink,
	Copy,
	Block,
}

pub struct TrackedFile {
	install_method: InstallMethod,
}

impl TrackedFile {
	pub fn get_install_method(&self) -> InstallMethod { self.install_method }
}

pub struct TrackedFiles {
	files: HashMap<String, TrackedFile>
}

impl TrackedFiles {
	pub fn add_file(&mut self, file: String, install_method: InstallMethod) {
		self.files.insert(file, TrackedFile { install_method });
	}

	pub fn get_file(&self, path: &str) -> Option<&TrackedFile> {
		self.files.get(path)
	}
}