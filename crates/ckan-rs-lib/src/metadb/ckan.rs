//! Various types associated with modules.

use std::{collections::{HashMap, HashSet}};
use serde::*;

/* CKAN */

/// A `.ckan` file containing mod info
/// We're not using serde for this thing because it's way to involved and limited. use `read_from_json` associated function instead
#[derive(Debug, Serialize, Deserialize)]
pub struct Ckan {
	/* Required Fields */
	pub spec_version: String,
	pub unique_id: relationship::ModUniqueIdentifier,
	pub name: String,
	/// Rust friendly alias for `abstract`
	pub blurb: String,
	/* one or many */
	pub author: Vec<String>,
	/* Required when `kind` is not `"metapackage"` or `"dlc"` */
	pub download: Option<String>,
	/* one or many */
	pub license: Vec<String>,
	
	/* Optional Fields */
	pub install: Vec<InstallDirective>,
	pub description: Option<String>,
	pub release_status: ReleaseStatus,
	pub ksp_version: KspVersionBounds,
	pub ksp_version_strict: Option<bool>,
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
	pub replaced_by: Option<ModuleDescriptor>,
	pub kind: Kind,
	pub provides: HashSet<String>,
	pub resources: HashMap<String, String>,
}

impl std::hash::Hash for Ckan {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.unique_id.hash(state);
	}
}

impl std::cmp::Ord for Ckan {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.unique_id.cmp(&other.unique_id)
	}
}

impl std::cmp::PartialOrd for Ckan {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl std::cmp::PartialEq for Ckan {
	fn eq(&self, other: &Self) -> bool {
		self.unique_id == other.unique_id
	}
}

impl std::cmp::Eq for Ckan {}

impl Ckan {
	/// Checks if the given modules conflict with each other
	pub fn do_modules_conflict(lhs: &Self, rhs: &Self) -> bool {
		let mut conflicts = false;
		for con in &lhs.conflicts {
			conflicts |= relationship::does_module_fulfill_relationship(rhs, con);
		}
		for con in &rhs.conflicts {
			conflicts |= relationship::does_module_fulfill_relationship(lhs, con);
		}
		conflicts
	}
}

/* CKAN Types */

mod version_bounds;
pub use version_bounds::VersionBounds;

mod ksp_version;
pub use ksp_version::KspVersion;
pub use ksp_version::KspVersionBounds;

mod mod_version;
pub use mod_version::ModVersion;

mod install;
pub use install::InstallDirective;

mod release;
pub use release::ReleaseStatus;

mod relationship;
pub use relationship::ModUniqueIdentifier;
pub use relationship::ModVersionBounds;
pub use relationship::Relationship;
pub use relationship::ModuleDescriptor;
pub use relationship::does_module_fulfill_relationship;
pub use relationship::does_module_provide_descriptor;
pub use relationship::does_module_match_descriptor;

mod kind;
pub use kind::Kind;

mod import;