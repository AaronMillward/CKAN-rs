//! Various types associated with modules.

use std::{collections::{HashMap, HashSet}};
use serde::*;

/* CKAN */

/// A `.ckan` file containing mod info
/// We're not using serde for this thing because it's way to involved and limited. use `read_from_json` associated function instead
#[derive(Debug, Eq, Serialize, Deserialize)]
pub struct Ckan {
	/* Required Fields */
	pub spec_version: String,
	pub identifier: String,
	pub name: String,
	pub r#abstract: String,
	/* one or many */
	pub author: Vec<String>,
	/* Required when `kind` is not `"metapackage"` or `"dlc"` */
	pub download: Option<String>,
	/* one or many */
	pub license: Vec<String>,
	pub version: ModVersion,
	
	/* Optional Fields */
	pub install: Vec<InstallDirective>,
	pub description: Option<String>,
	pub release_status: ReleaseStatus,
	pub ksp_version: Option<KspVersion>,
	pub ksp_version_min: Option<KspVersion>,
	pub ksp_version_max: Option<KspVersion>,
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
		/* XXX: As far as I'm aware these are the unique identifiers for modules */
		self.identifier.hash(state);
		self.version.hash(state);
	}
}

impl std::cmp::Ord for Ckan {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match self.identifier.cmp(&other.identifier) {
			core::cmp::Ordering::Equal => {}
			ord => return ord,
		}
		/* XXX: Maybe release status should affect sort order? */
		// match self.release_status.partial_cmp(&other.release_status) {
		// 	Some(core::cmp::Ordering::Equal) => {}
		// 	ord => return ord,
		// }
		self.version.cmp(&other.version)
	}
}

impl std::cmp::PartialEq for Ckan {
	fn eq(&self, other: &Self) -> bool {
		self.identifier == other.identifier &&
		self.version == other.version
	}
}

impl std::cmp::PartialOrd for Ckan {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

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

mod ksp_version;
pub use ksp_version::KspVersion;

mod mod_version;
pub use mod_version::ModVersion;

mod install;
pub use install::InstallDirective;

mod release;
pub use release::ReleaseStatus;

mod relationship;
pub use relationship::Relationship;
pub use relationship::ModuleDescriptor;
pub use relationship::does_module_fulfill_relationship;
pub use relationship::does_module_match_descriptor;

mod kind;
pub use kind::Kind;

mod import;