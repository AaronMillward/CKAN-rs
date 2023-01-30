//! Used to track what files have been installed to a game directory.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TrackedFiles {
	/* TODO: In the future we could use a tuple in this Vec instead to store additional data such as method or reason. */
	files: HashMap<crate::metadb::ckan::PackageIdentifier, Vec<String>>
}

impl TrackedFiles {
	pub fn add_file(&mut self, package: &crate::metadb::ckan::PackageIdentifier, file: String) {
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
}