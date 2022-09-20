use std::{collections::HashMap};
use serde::{*, de::DeserializeOwned};

// Thanks to
// https://github.com/Mingun/ksc-rs/blob/8532f701e660b07b6d2c74963fdc0490be4fae4b/src/parser.rs#L18-L42
// https://github.com/serde-rs/serde/issues/1907
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OneOrMany<T> {
	One(T),
	Vec(Vec<T>),
}
impl<T> Default for OneOrMany<T> where T: Default {
	fn default() -> Self {
		OneOrMany::One(T::default())
	}
}
impl<T> From<OneOrMany<T>> for Vec<T> {
	fn from(from: OneOrMany<T>) -> Self {
		match from {
			OneOrMany::One(val) => vec![val],
			OneOrMany::Vec(vec) => vec,
		}
	}
}

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
	}

	impl InstallDirective {
		pub fn from_json(v: &serde_json::Value) -> crate::Result<Vec<Self>> {
			use crate::Error::ParseError;

			let mut directives = Vec::<Self>::new();

			if let Some(arr) = v.as_array() {
				for elem in arr {
					if let Some(obj) = elem.as_object() {
						let directive = InstallDirective {
							source: {
								if let Some(f) = obj.get("file") {
									SourceDirective::File(
										f.as_str().ok_or_else(|| ParseError("file source directive must be a string".to_string()))?.to_string()
									)
								} else if let Some(f) = obj.get("find") {
									SourceDirective::Find(
										f.as_str().ok_or_else(|| ParseError("find source directive must be a string".to_string()))?.to_string()
									)
								} else if let Some(f) = obj.get("find_regexp") {
									SourceDirective::FindRegExp(
										f.as_str().ok_or_else(|| ParseError("find_regexp source directive must be a string".to_string()))?.to_string()
									)
								} else {
									return Err(ParseError("install has no valid source directive".to_string()));
								}
							},

							install_to: {
								if let Some(f) = obj.get("install_to") {
									f.as_str().ok_or_else(|| ParseError("destination directive must be a string".to_string()))?.to_string()
								} else {
									return Err(ParseError("install has no destination directive".to_string()));
								}
							},

							additional: {
								let mut add = Vec::<OptionalDirective>::new();
								/* The spec doesn't mention specifically but I'm pretty sure each directive can only turn up once */
								if let Some(f) = obj.get("as") {
									add.push(OptionalDirective::As(f.as_str().ok_or_else(|| ParseError("as directive must be a string".to_string()))?.to_string()));
								}
								if obj.get("filter").is_some() {
									add.push(OptionalDirective::Filter(super::get_one_or_many_string(obj, "filter")?));
								}
								if obj.get("filter_regexp").is_some() {
									add.push(OptionalDirective::FilterRegExp(super::get_one_or_many_string(obj, "filter_regexp")?));
								}
								if obj.get("include_only").is_some() {
									add.push(OptionalDirective::IncludeOnly(super::get_one_or_many_string(obj, "include_only")?));
								}
								if obj.get("include_only_regexp").is_some() {
									add.push(OptionalDirective::IncludeOnlyRegExp(super::get_one_or_many_string(obj, "include_only_regexp")?));
								}
								if let Some(f) = obj.get("find_matches_files") {
									add.push(OptionalDirective::FindMatchesFiles(f.as_bool().ok_or_else(|| ParseError("find_matches_files directive must be a bool".to_string()))?));
								}

								add
							}
						};
						directives.push(directive);
					} else {
						return Err(ParseError("array elements must be objects".to_string()));
					}
				}
			} else {
				return Err(ParseError("must be array".to_string()));
			}

			Ok(directives)
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
impl rusqlite::ToSql for ReleaseStatus {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(rusqlite::types::ToSqlOutput::from(match self {
			ReleaseStatus::Stable => 0u8,
			ReleaseStatus::Testing => 1,
			ReleaseStatus::Development => 2,
		}))
	}
}


/* Relationships */

mod relationship {
	use serde::*;

	#[derive(Debug, Serialize, Deserialize)]
	pub struct RelationshipEntry {
		name: String,
		version: Option<String>,
		min_version: Option<String>,
		max_version: Option<String>,
	}
	impl RelationshipEntry {
		pub fn from_json(v: &serde_json::Value) -> crate::Result<Self> {
			use crate::Error::ParseError;
			Ok(RelationshipEntry {
				name: {
					v.get("name")
						.ok_or_else(|| ParseError("JSON has no name field".to_string()))?
						.as_str().ok_or_else(|| ParseError("name must be a string".to_string()))?.to_string()
				},
				version: v.get("version").and_then(|v| v.as_str().map(|s| s.to_string())),
				max_version: v.get("max_version").and_then(|v| v.as_str().map(|s| s.to_string())),
				min_version: v.get("min_version").and_then(|v| v.as_str().map(|s| s.to_string())),
			})
		}
	}
	
	#[derive(Debug, Serialize, Deserialize)]
	pub enum Relationship {
		AnyOf(Vec<RelationshipEntry>),
		One(RelationshipEntry),
	}

	pub fn from_json(v: &serde_json::Value) -> crate::Result<Vec<Relationship>> {
		use crate::Error::ParseError;

		let mut relationships = Vec::<Relationship>::new();

		if let Some(arr) = v.as_array() {
			for elem in arr {
				/* Process each relationship */
				if let Some(obj) = elem.as_object() {
					let relationship = {
						/* any_of */
						if let Some(f) = obj.get("any_of") {
							if let Some(arr) = f.as_array() {
								let mut ships = Vec::<RelationshipEntry>::new();
								for o in arr {
									if o.is_object() {
										if let Ok(val) = RelationshipEntry::from_json(o) {
											ships.push(val);
										}
									} else {
										return Err(ParseError("any_of array must contain only objects".to_string()));
									}
								}
								Relationship::AnyOf(ships)
							} else {
								return Err(ParseError("any_of constraint must be an array".to_string()));
							}
						/* single */
						} else if obj.get("name").is_some() {
							Relationship::One(RelationshipEntry::from_json(elem)?)
						} else {
							return Err(ParseError("relationship object must be a relationship or any_of constraint".to_string()));
						}
					};
					relationships.push(relationship);
				} else {
					return Err(ParseError("array elements must be objects".to_string()));
				}
			}
		} else {
			return Err(ParseError("must be array".to_string()));
		}

		Ok(relationships)
	}
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
#[derive(Debug, Default)]
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
	pub version: String,
	
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
	pub download_size: Option<u64>, /* *Really* Don't use anything lower then 64 here, 32 is only 4gb max size */
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

impl Ckan {
	pub fn read_from_json(v: serde_json::Value) -> crate::Result<Self> {
		use crate::Error::ParseError as ParseError;
		use serde_json::*;
		
		fn get_val<T>(map: &Map<String, Value>, key: &str) -> crate::Result<T> 
		where T: DeserializeOwned {
			Ok(
				serde_json::from_value(map.get(key).unwrap_or(&Value::Null).to_owned())?
			)
		}

		let obj = v.as_object().ok_or_else(|| ParseError("JSON is not an object".to_string()))?;
		Ok( Ckan {
			spec_version: {
				match obj.get("spec_version").unwrap_or(&Value::Null) {
					Value::Number(v) => v.to_string(),
					Value::String(v) => v.to_owned(),
					_ => return Err(ParseError("invalid type".to_string())),
				}
			},
			identifier: get_val(obj, "identifier")?,
			name: get_val(obj, "name")?,
			r#abstract: get_val(obj, "abstract")?,
			author: get_one_or_many_string(obj, "author")?,
			download: {
				match obj.get("download") {
					Some(v) => {
						match v {
							Value::String(v) => {
								Some(v.clone())
							},
							_ => return Err(ParseError("invalid type, expected string for download".to_string())),
						}
					},
					None => None,
				}
			},
			license: get_one_or_many_string(obj, "license")?,
			version: get_val(obj, "version")?,

			/* Optionals */
			install: {
				if let Some(v) = obj.get("install") {
					install::InstallDirective::from_json(v).unwrap_or_default()
				} else {
					Vec::<install::InstallDirective>::new()
				}
			},
			description: get_val(obj, "description").ok(),
			release_status: {
				match obj.get("release_status") {
					Some(v) => {
						match v {
							Value::String(v) => {
								if v == "stable" {
									ReleaseStatus::Stable
								} else if v == "testing" {
									ReleaseStatus::Testing
								} else if v == "development" {
									ReleaseStatus::Development
								} else {
									return Err(ParseError("unknown release_status".to_string()))
								}
							},
							_ => return Err(ParseError("invalid type, expected string for release_status".to_string())),
						}
					},
					None => ReleaseStatus::Stable,
				}
			},
			ksp_version: get_val(obj, "ksp_version").ok(),
			ksp_version_min: get_val(obj, "ksp_version_min").ok(),
			ksp_version_max: get_val(obj, "ksp_version_max").ok(),
			ksp_version_strict: serde_json::from_value(obj.get("ksp_version_strict").cloned().unwrap_or(Value::Bool(true))).map_err(|_| ParseError("ksp_version_strict must be a boolean".to_string()))?,
			tags: get_one_or_many_string(obj, "tags").ok(), /* This does work */
			localizations: get_one_or_many_string(obj, "localizations").ok(),
			download_size: get_val(obj, "download_size").ok(),
			download_hash_sha1: {
				/* Looks bad but the functional equivalent looks worse */
				let mut res = None;
				if let Some(h) = obj.get("download_hash") {
					if let Some(o) = h.as_object() {
						if let Some(hash) = o.get("sha1") {
							if let Some(s) = hash.as_str() {
								res = s.as_bytes().try_into().ok();
							}
						}
					}
				}
				res
			},
			download_hash_sha256: {
				let mut res = None;
				if let Some(h) = obj.get("download_hash") {
					if let Some(o) = h.as_object() {
						if let Some(hash) = o.get("sha256") {
							if let Some(s) = hash.as_str() {
								res = s.as_bytes().try_into().ok();
							}
						}
					}
				}
				res
			},
			download_content_type: get_val(obj, "download_content_type").ok(),
			install_size: get_val(obj, "install_size").ok(),
			release_date: get_val(obj, "release_date").ok(),
			depends: obj.get("depends").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			recommends: obj.get("recommends").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			suggests: obj.get("suggests").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			supports: obj.get("supports").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			conflicts: obj.get("conflicts").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			replaced_by: obj.get("replaced-by").map(|v| relationship::RelationshipEntry::from_json(v).expect("couldn't read relationship from JSON")),
			kind: get_val(obj, "kind").unwrap_or_default(),
			resources: get_val(obj, "resources").unwrap_or_default(),
		})
	}
}