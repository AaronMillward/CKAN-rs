//! Module containing the various types associated with CKAN files.

use std::{collections::HashMap};
use serde::{*, de::DeserializeOwned};

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
mod sql;


/* CKAN Types */

#[derive(Debug, Serialize, Deserialize)]
pub struct ModVersion {
	epoch: i32,
	mod_version: String,
}
impl ModVersion {
	pub fn new(version: String) -> crate::Result<Self> {
		/* FIXME: mod_version can be *any* string so this method assumes mod_version doesn't contain a ':' */
		let spl: Vec<&str> = version.splitn(2,':').collect();
		Ok(ModVersion {
			epoch: {
				spl[0].parse::<i32>().unwrap_or(0) /* FIXME: We assume spl[0] is not just a number */
			},
			mod_version: {
				spl[spl.len() - 1].to_string()
			}
		})
	}
}
impl TryFrom<String> for ModVersion {
	type Error = crate::Error;
	fn try_from(value: String) -> Result<Self, Self::Error> { Self::new(value) }
}
impl PartialEq for ModVersion {
	fn eq(&self, other: &Self) -> bool {
		self.mod_version == other.mod_version
	}
}
impl PartialOrd for ModVersion {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		if self.epoch != other.epoch {
			self.epoch.partial_cmp(&other.epoch)
		} else {
			/* TODO:FIXME: the spec is very wordy about how this is compaired, just doing something basic for now */
			let mut ord = std::cmp::Ordering::Equal;
			for (lhs,rhs) in self.mod_version.chars().zip(other.mod_version.chars()) {
				let res = lhs.partial_cmp(&rhs).unwrap();
				match res {
					std::cmp::Ordering::Equal => continue,
					_ => ord = res,
				}
			}
			Some(ord)
		}
	}
}

/* Install */

mod install {
	use serde::*;

	#[derive(Debug, Serialize, Deserialize)]
	#[serde(untagged)]
	pub enum SourceDirective {
		#[serde(rename = "file")]
		File(String),
		#[serde(rename = "find")]
		Find(String),
		#[serde(rename = "find_regexp")]
		FindRegExp(String),
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub enum OptionalDirective {
		As(String),
		Filter(Vec<String>),
		FilterRegExp(Vec<String>),
		IncludeOnly(Vec<String>),
		IncludeOnlyRegExp(Vec<String>),
		FindMatchesFiles(bool),
	}

	#[derive(Debug, Serialize, Deserialize)]
	pub struct InstallDirective {
		source: SourceDirective,
		install_to: String,
		additional: Vec<OptionalDirective>,
	} impl InstallDirective {
		pub fn new(source: SourceDirective, install_to: String, additional: Vec<OptionalDirective>) -> Self {
			Self { source, install_to, additional }
		}
	}
}

/* Release */

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseStatus {
	Stable,
	Testing,
	Development,
}
impl Default for ReleaseStatus { fn default() -> Self { Self::Stable } }

/* Relationships */

pub(crate) mod relationship {
	use serde::*;
	use super::*;

	#[derive(Debug, Serialize, Deserialize)]
	pub struct RelationshipEntry {
		name: String,
		version: Option<ModVersion>,
		min_version: Option<ModVersion>,
		max_version: Option<ModVersion>,
	} impl RelationshipEntry {
		pub fn new(name: String, version: Option<String>, min_version: Option<String>, max_version: Option<String> ) -> Self {
			/* TODO: Verify input */
			
			Self {
				name,
				version: version.map(|v| ModVersion::new(v).unwrap()),
				min_version: min_version.map(|v| ModVersion::new(v).unwrap()),
				max_version: max_version.map(|v| ModVersion::new(v).unwrap()),
			}
		}
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub enum Relationship {
		AnyOf(Vec<RelationshipEntry>),
		One(RelationshipEntry),
	}

	pub use super::import::relationship_from_json as from_json;
}

/* Kind */

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
	Package,
	MetaPackage,
	Dlc,
}
impl Default for Kind { fn default() -> Self { Self::Package } }

/* CKAN */

/// A `.ckan` file containing mod info
/// Read more about the spec [here](https://github.com/KSP-CKAN/CKAN/blob/master/Spec.md)
/// We're not using serde for this thing because it's way to involved and limited. use `read_from_json` associated function instead
#[derive(Debug)]
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
	pub install: Vec<install::InstallDirective>,
	pub description: Option<String>,
	pub release_status: ReleaseStatus,
	pub ksp_version: Option<String>,
	pub ksp_version_min: Option<String>,
	pub ksp_version_max: Option<String>,
	pub ksp_version_strict: Option<bool>,
	pub tags: Option<Vec<String>>,
	pub localizations: Option<Vec<String>>,
	pub download_size: Option<u64>, /* *Really* Don't use anything lower than 64 here, 32 is only 4gb max size */
	pub download_hash_sha1: Option<[u8; 40]>,
	pub download_hash_sha256: Option<[u8; 64]>,
	pub download_content_type: Option<String>,
	pub install_size: Option<u64>,
	pub release_date: Option<String>,
	pub depends: Vec<relationship::Relationship>,
	pub recommends: Vec<relationship::Relationship>,
	pub suggests: Vec<relationship::Relationship>,
	pub supports: Vec<relationship::Relationship>,
	pub conflicts: Vec<relationship::Relationship>,
	pub replaced_by: Option<relationship::RelationshipEntry>,
	pub kind: Kind,
	pub resources: HashMap<String, String>,
}