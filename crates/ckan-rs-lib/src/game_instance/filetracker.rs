//! Used to track what files have been installed to a game directory.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TrackedFiles {
	/* TODO: In the future we could use a tuple in this Vec instead to store additional data such as method or reason. */
	files: HashMap<crate::metadb::package::PackageIdentifier, Vec<String>>
}

impl TrackedFiles {
	pub fn add_file(&mut self, package: &crate::metadb::package::PackageIdentifier, file: String) {
		let existing = self.files.get_mut(package);

		if let Some(val) = existing {
			val.push(file);
		} else {
			self.files.insert(package.clone(), vec![file]);
		}
	}

	pub fn clear(&mut self) {
		self.files.clear();
	}

	pub fn get_all_files(&self) -> Vec<&str> {
		let mut v = Vec::<_>::new();
		for f in self.files.values() {
			for s in f {
				v.push(s.as_str());
			}
		}
		
		v
	}
}