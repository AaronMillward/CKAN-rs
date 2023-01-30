//! Functions and methods for reading CKAN types from JSON

use serde::de::DeserializeOwned;
use try_map::FallibleMapExt;
use super::*;

fn get_one_or_many_string(obj: &serde_json::Value, key: &str) -> crate::Result<Vec<String>> {
	let v = obj.get(key).ok_or_else(|| crate::Error::Parse(format!("key {} missing", key)))?;
	match v {
		serde_json::Value::Array(_) => Ok(serde_json::from_value(v.to_owned())?),
		serde_json::Value::String(v) => {
			Ok(vec![v.to_owned()])
		},
		_ => Err(crate::Error::Parse(format!("key {} is not a string or array", key))),
	}
}

impl install::InstallDirective {
	pub fn from_json(v: &serde_json::Value) -> crate::Result<Vec<Self>> {
		use crate::Error::Parse;
		use install::*;

		let mut directives = Vec::<Self>::new();

		if !v.is_array() {
			return Err(Parse("value must be an array".to_string()));
		}
		for obj in v.as_array().unwrap() {
			if !obj.is_object() {
				return Err(Parse("array elements must be objects".to_string()));
			}
			let directive = InstallDirective::new(
				{
					if let Some(f) = obj.get("file") {
						SourceDirective::File(
							f.as_str().ok_or_else(|| Parse("file source directive must be a string".to_string()))?.to_string()
						)
					} else if let Some(f) = obj.get("find") {
						SourceDirective::Find(
							f.as_str().ok_or_else(|| Parse("find source directive must be a string".to_string()))?.to_string()
						)
					} else if let Some(f) = obj.get("find_regexp") {
						SourceDirective::FindRegExp(
							f.as_str().ok_or_else(|| Parse("find_regexp source directive must be a string".to_string()))?.to_string()
						)
					} else {
						return Err(Parse("install has no valid source directive".to_string()));
					}
				},

				{
					if let Some(f) = obj.get("install_to") {
						f.as_str().ok_or_else(|| Parse("destination directive must be a string".to_string()))?.to_string()
					} else {
						return Err(Parse("install has no destination directive".to_string()));
					}
				},

				{
					let mut add = Vec::<OptionalDirective>::new();
					/* The spec doesn't mention specifically but I'm pretty sure each directive can only turn up once */
					if let Some(f) = obj.get("as") {
						add.push(OptionalDirective::As(f.as_str().ok_or_else(|| Parse("as directive must be a string".to_string()))?.to_string()));
					}
					if obj.get("filter").is_some() {
						add.push(OptionalDirective::Filter(get_one_or_many_string(obj, "filter")?));
					}
					if obj.get("filter_regexp").is_some() {
						add.push(OptionalDirective::FilterRegExp(get_one_or_many_string(obj, "filter_regexp")?));
					}
					if obj.get("include_only").is_some() {
						add.push(OptionalDirective::IncludeOnly(get_one_or_many_string(obj, "include_only")?));
					}
					if obj.get("include_only_regexp").is_some() {
						add.push(OptionalDirective::IncludeOnlyRegExp(get_one_or_many_string(obj, "include_only_regexp")?));
					}
					if let Some(f) = obj.get("find_matches_files") {
						add.push(OptionalDirective::FindMatchesFiles(f.as_bool().ok_or_else(|| Parse("find_matches_files directive must be a bool".to_string()))?));
					}

					add
				}
			);
			directives.push(directive);
		}

		Ok(directives)
	}
}

impl mod_version::PackageVersion {
	pub fn from_json(v: &serde_json::Value) -> crate::Result<Self> {
		use crate::Error::Parse;
		v.as_str()
			.ok_or_else(|| Parse("version must be a string".to_string()))
			.and_then(|s|
				PackageVersion::new(s).map_err(|_| Parse("version string can't be read as a version".to_string()))
			)
	}
}

impl relationship::PackageDescriptor {
	pub fn from_json(v: &serde_json::Value) -> crate::Result<Self> {
		use crate::Error::Parse;
		Ok(relationship::PackageDescriptor::new(
			{
				v.get("name")
					.ok_or_else(|| Parse("JSON has no name field".to_string()))?
					.as_str().ok_or_else(|| Parse("name must be a string".to_string()))?.to_string()
			},
			relationship::PackageVersionBounds::new(
				v.get("version").try_map(PackageVersion::from_json)?,
				v.get("min_version").try_map(PackageVersion::from_json)?,
				v.get("max_version").try_map(PackageVersion::from_json)?
			)?
		))
	}
}

pub fn relationship_from_json(v: &serde_json::Value) -> crate::Result<Vec<relationship::Relationship>> {
	use crate::Error::Parse;
	use relationship::*;

	let mut relationships = Vec::<Relationship>::new();

	if let Some(arr) = v.as_array() {
		for elem in arr {
			/* Process each relationship */
			if let Some(obj) = elem.as_object() {
				let relationship = {
					/* any_of */
					if let Some(f) = obj.get("any_of") {
						if let Some(arr) = f.as_array() {
							let mut ships = Vec::<PackageDescriptor>::new();
							for o in arr {
								if o.is_object() {
									if let Ok(val) = PackageDescriptor::from_json(o) {
										ships.push(val);
									}
								} else {
									return Err(Parse("any_of array must contain only objects".to_string()));
								}
							}
							Relationship::AnyOf(ships)
						} else {
							return Err(Parse("any_of constraint must be an array".to_string()));
						}
					/* single */
					} else if obj.get("name").is_some() {
						Relationship::One(PackageDescriptor::from_json(elem)?)
					} else {
						return Err(Parse("relationship object must be a relationship or any_of constraint".to_string()));
					}
				};
				relationships.push(relationship);
			} else {
				return Err(Parse("array elements must be objects".to_string()));
			}
		}
	} else {
		return Err(Parse("must be array".to_string()));
	}

	Ok(relationships)
}

impl Package {
	pub fn read_from_json(v: serde_json::Value) -> crate::Result<Self> {
		use crate::Error::Parse;
		use serde_json::*;

		fn get_val<T>(object: &Value, key: &str) -> crate::Result<T>
		where T: DeserializeOwned {
			object.get(key)
				.ok_or_else(|| Parse(format!("Failed to get key: {}", key)))
				.and_then(|v| serde_json::from_value::<T>(v.to_owned())
					.map_err(crate::Error::from)
				)
		}

		fn get_val_optional<T>(object: &Value, key: &str) -> crate::Result<Option<T>>
		where T: DeserializeOwned {
			if let Some(v) = object.get(key) {
				serde_json::from_value::<T>(v.to_owned())
					.map(|r| Some(r))
					.map_err(crate::Error::from)
			} else {
				Ok(None)
			}
		}

		/* FIXME: Lots of panics and error ignorance */

		let obj = &v;
		Ok( Package {
			spec_version: {
				match obj.get("spec_version").ok_or_else(|| Parse("`spec_version` is missing".to_string()))? {
					Value::Number(v) => v.to_string(),
					Value::String(v) => v.to_owned(),
					_ => return Err(Parse("invalid type".to_string())),
				}
			},
			identifier: relationship::PackageIdentifier {
				identifier: get_val(obj, "identifier")?,
				version: obj.get("version")
					.ok_or_else(|| Parse("`version` is missing".to_string()))
					.and_then(|v| v.as_str().ok_or_else(|| Parse("`version` must be a string".to_string())))
					.and_then(PackageVersion::new)
					?,
			},
			name: get_val(obj, "name")?,
			blurb: get_val(obj, "abstract")?,
			author: get_one_or_many_string(obj, "author")?,
			download: {
				/* TODO: Error when key is wrong type */
				/* TODO: Check `kind` to see if absense is an error */
				obj.get("download")
					.and_then(|v| v.as_str())
					.map(|s| s.to_string())
			},
			license: get_one_or_many_string(obj, "license")?,

			/* Optionals */
			install: {
				if let Some(v) = obj.get("install") {
					install::InstallDirective::from_json(v)?
				} else {
					Vec::<install::InstallDirective>::new()
				}
			},
			description: get_val_optional(obj, "description")?,
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
									return Err(Parse("unknown release_status".to_string()))
								}
							},
							_ => return Err(Parse("invalid type, expected string for release_status".to_string())),
						}
					},
					None => ReleaseStatus::Stable,
				}
			},
			ksp_version: {
				VersionBounds::new(
					get_val_optional::<String>(obj, "ksp_version").map(|v| v.map(|s| KspVersion::from(s.as_str())))?,
					get_val_optional::<String>(obj, "ksp_version_min").map(|v| v.map(|s| KspVersion::from(s.as_str())))?,
					get_val_optional::<String>(obj, "ksp_version_max").map(|v| v.map(|s| KspVersion::from(s.as_str())))?,
				)?
			},
			ksp_version_strict: serde_json::from_value(obj.get("ksp_version_strict").cloned().unwrap_or(Value::Bool(true))).map_err(|_| Parse("ksp_version_strict must be a boolean".to_string()))?,
			tags: get_one_or_many_string(obj, "tags").ok(), /* This does work */
			localizations: get_one_or_many_string(obj, "localizations").ok(),
			download_size: get_val_optional(obj, "download_size")?,
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
			download_content_type: get_val_optional(obj, "download_content_type")?,
			install_size: get_val_optional(obj, "install_size")?,
			release_date: get_val_optional(obj, "release_date")?,
			depends: obj.get("depends").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			recommends: obj.get("recommends").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			suggests: obj.get("suggests").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			supports: obj.get("supports").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			conflicts: obj.get("conflicts").map_or_else(Vec::<relationship::Relationship>::default, |v| relationship::from_json(v).expect("couldn't read relationship from JSON")),
			replaced_by: obj.get("replaced-by").map(|v| relationship::PackageDescriptor::from_json(v).expect("couldn't read relationship from JSON")),
			kind: get_val(obj, "kind").unwrap_or_default(),
			provides: {
				obj.get("provides").and_then(|value|
					value.as_array()
					.map(|array| array.iter()
					.map(|e| e.as_str().expect("`provides` elements must be strings").to_string())
					.collect::<HashSet<_>>())
				).unwrap_or_default()
			},
			resources: get_val(obj, "resources").unwrap_or_default(), /* FIXME: doesn't handle read errors */
		})
	}
}

