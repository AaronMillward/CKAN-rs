//! Functions and methods for reading CKAN types from JSON

use super::*;

impl install::InstallDirective {
	pub fn from_json(v: &serde_json::Value) -> crate::Result<Vec<Self>> {
		use crate::Error::ParseError;
		use install::*;

		let mut directives = Vec::<Self>::new();

		if let Some(arr) = v.as_array() {
			for elem in arr {
				if let Some(obj) = elem.as_object() {
					let directive = InstallDirective::new(
						{
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

						{
							if let Some(f) = obj.get("install_to") {
								f.as_str().ok_or_else(|| ParseError("destination directive must be a string".to_string()))?.to_string()
							} else {
								return Err(ParseError("install has no destination directive".to_string()));
							}
						},

						{
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
					);
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

impl relationship::RelationshipEntry {
	pub fn from_json(v: &serde_json::Value) -> crate::Result<Self> {
		use crate::Error::ParseError;
		Ok(relationship::RelationshipEntry::new(
			{
				v.get("name")
					.ok_or_else(|| ParseError("JSON has no name field".to_string()))?
					.as_str().ok_or_else(|| ParseError("name must be a string".to_string()))?.to_string()
			},
			v.get("version").and_then(|v| v.as_str().map(|s| s.to_string())),
			v.get("max_version").and_then(|v| v.as_str().map(|s| s.to_string())),
			v.get("min_version").and_then(|v| v.as_str().map(|s| s.to_string())),
		))
	}
}

pub fn relationship_from_json(v: &serde_json::Value) -> crate::Result<Vec<relationship::Relationship>> {
	use crate::Error::ParseError;
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
			provides: {
				obj.get("provides").and_then(|value|
					value.as_array().and_then(|array|
						Some(
							array.iter().map(|e| e.as_str().expect("`provides` elements must be strings").to_string()).collect::<Vec<_>>()
						)
					)
				).unwrap_or_default()
			},
			resources: get_val(obj, "resources").unwrap_or_default(),
		})
	}
}

