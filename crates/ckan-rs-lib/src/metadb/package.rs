//! Various types associated with packages.

use std::{collections::{HashMap, HashSet}};
use serde::*;

/* CKAN */

/// A `.ckan` file containing mod info.
/// 
/// We use the term "Package" instead of "Module" due to the overlap with rust's keywords.
/* NOTE: We don't use serde's deserialize to import the .ckan files because it's way to involved and limited. use `read_from_json` associated function instead. */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
	/* Required Fields */
	pub spec_version: String,
	pub identifier: relationship::PackageIdentifier,
	pub name: String,
	/// Rust friendly alias for `abstract`.
	pub blurb: String,
	/* one or many */
	pub author: Vec<String>,
	/* Required when `kind` is not `"metapackage"` or `"dlc"` */
	pub download: Option<String>,
	/* one or many */
	pub license: Vec<String>,
	
	/* Optional Fields */
	pub install: Vec<install::InstallDirective>,
	pub description: Option<String>,
	pub release_status: ReleaseStatus,
	pub ksp_version: ksp_version::KspVersionBounds,
	pub ksp_version_strict: bool,
	pub tags: Option<Vec<String>>,
	pub localizations: Option<Vec<String>>,
	pub download_size: Option<u64>, /* *Really* Don't use anything lower than 64 here, 32 is only 4gb max size */
	pub download_hash_sha1: Option<Vec<u8>>,
	pub download_hash_sha256: Option<Vec<u8>>,
	pub download_content_type: Option<String>,
	pub install_size: Option<u64>,
	pub release_date: Option<String>,
	pub depends: Vec<Relationship>,
	pub recommends: Vec<Relationship>,
	pub suggests: Vec<Relationship>,
	pub supports: Vec<Relationship>,
	pub conflicts: Vec<Relationship>,
	pub replaced_by: Option<PackageDescriptor>,
	pub kind: Kind,
	pub provides: HashSet<String>,
	pub resources: HashMap<String, String>,
}

impl std::hash::Hash for Package {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.identifier.hash(state);
	}
}

impl std::cmp::Ord for Package {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.identifier.cmp(&other.identifier)
	}
}

impl std::cmp::PartialOrd for Package {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl std::cmp::PartialEq for Package {
	fn eq(&self, other: &Self) -> bool {
		self.identifier == other.identifier
	}
}

impl std::cmp::Eq for Package {}

impl AsRef<PackageIdentifier> for Package {
	fn as_ref(&self) -> &PackageIdentifier {
		&self.identifier
	}
}

impl Package {
	/// Checks if the given packages conflict with each other.
	pub fn do_packages_conflict(lhs: &Self, rhs: &Self) -> bool {
		let mut conflicts = false;
		for con in &lhs.conflicts {
			conflicts |= relationship::does_package_fulfill_relationship(rhs, con);
		}
		for con in &rhs.conflicts {
			conflicts |= relationship::does_package_fulfill_relationship(lhs, con);
		}
		conflicts
	}
}

/* CKAN Types */

mod version_bounds;
pub use version_bounds::VersionBounds;

mod ksp_version;
pub use ksp_version::KspVersionReal;
pub use ksp_version::KspVersionBounds;

mod package_version;
pub use package_version::PackageVersion;
pub use package_version::PackageVersionBounds;

pub mod install;


mod relationship;
pub use relationship::PackageIdentifier;
pub use relationship::PackageDescriptor;
pub use relationship::Relationship;
pub use relationship::does_package_fulfill_relationship;
pub use relationship::does_package_provide_descriptor;
pub use relationship::does_package_match_descriptor;

/// The stability of a package.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReleaseStatus {
	#[default] Stable,
	Testing,
	Development,
}

/* TODO: Move Kind-dependent info (such as download url) into this enum's variants */
/// The type of a package.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
	/// A normal installable module.
	#[default] Package,
	/// A distributable .ckan file that has relationships to other mods while having no download of its own.
	MetaPackage,
	/// A paid expansion from SQUAD, which CKAN can detect but not install. Also has no download.
	DLC,
}

mod import;