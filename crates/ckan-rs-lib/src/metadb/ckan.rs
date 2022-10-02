//! Module containing the various types associated with CKAN files.

use std::{collections::{HashMap, HashSet}};
use serde::*;

/* CKAN */

/// A `.ckan` file containing mod info
/// Read more about the spec [here](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)
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
	pub replaced_by: Option<RelationshipEntry>,
	pub kind: Kind,
	pub provides: HashSet<String>,
	pub resources: HashMap<String, String>,
}

impl std::cmp::PartialEq for Ckan {
	fn eq(&self, other: &Self) -> bool {
		self.identifier == other.identifier &&
		self.name == other.name &&
		self.version == other.version &&
		self.release_status == other.release_status
	}
}

impl std::cmp::PartialOrd for Ckan {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match self.identifier.partial_cmp(&other.identifier) {
			Some(core::cmp::Ordering::Equal) => {}
			ord => return ord,
		}
		/* XXX: Maybe release status should affect sort order? */
		// match self.release_status.partial_cmp(&other.release_status) {
		// 	Some(core::cmp::Ordering::Equal) => {}
		// 	ord => return ord,
		// }
		match self.version.partial_cmp(&other.version) {
			Some(core::cmp::Ordering::Equal) => {}
			ord => return ord,
		}

		Some(core::cmp::Ordering::Equal)
	}
}

impl std::hash::Hash for Ckan {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		/* XXX: As far as I'm aware these are the unique identifiers for modules */
		self.identifier.hash(state);
		self.version.hash(state);
	}
}

impl Ckan {
	/// Checks if the given modules conflict with each other
	pub fn do_modules_conflict(lhs: &Self, rhs: &Self) -> bool {
		let mut conflicts = false;
		for con in &lhs.conflicts {
			conflicts &= relationship::does_module_fulfill_relationship(con, rhs);
		}
		for con in &rhs.conflicts {
			conflicts &= relationship::does_module_fulfill_relationship(con, lhs);
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

pub(crate) mod relationship;
pub use relationship::Relationship;
pub use relationship::RelationshipEntry;

mod kind;
pub use kind::Kind;


/* Conversions */

fn get_one_or_many_string(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> crate::Result<Vec<String>> {
	let v = map.get(key).ok_or_else(|| crate::Error::ParseError(format!("key {} missing", key)))?;
	match v {
		serde_json::Value::Array(_) => Ok(serde_json::from_value(v.to_owned())?),
		serde_json::Value::String(v) => {
			Ok(vec![v.to_owned()])
		},
		_ => Err(crate::Error::ParseError(format!("key {} is not a string or array", key))),
	}
}

mod import;