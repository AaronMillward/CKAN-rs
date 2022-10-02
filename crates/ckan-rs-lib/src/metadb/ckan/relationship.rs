use serde::*;
use super::{*, mod_version::ModVersion};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipEntry {
	name: String,
	version: Option<ModVersion>,
	min_version: Option<ModVersion>,
	max_version: Option<ModVersion>,
} impl RelationshipEntry {
	pub fn new(name: String, version: Option<String>, min_version: Option<String>, max_version: Option<String> ) -> Self {
		/* TODO: Don't panic */
		if version.is_none() && min_version.is_none() && max_version.is_none() { panic!("relationship entry without any version requirements") }
		
		Self {
			name,
			version: version.map(|v| ModVersion::new(v).unwrap()),
			min_version: min_version.map(|v| ModVersion::new(v).unwrap()),
			max_version: max_version.map(|v| ModVersion::new(v).unwrap()),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relationship {
	AnyOf(Vec<RelationshipEntry>),
	One(RelationshipEntry),
}

pub fn does_module_fulfill_relationship(relationship: &Relationship, module: &Ckan) -> bool {
	let v = match relationship {
		Relationship::AnyOf(v) => v.iter().collect(),
		Relationship::One(rel) => vec![rel],
	};
	
	let mut does_not_match = false;
	for rel in v {
		if module.identifier == rel.name {
			if let Some(version) = &rel.version {
				does_not_match &= &module.version == version;
			}
			if let Some(min_version) = &rel.min_version {
				does_not_match &= &module.version < min_version;
			}
			if let Some(max_version) = &rel.max_version {
				does_not_match &= &module.version > max_version;
			}
		}
	}
	!does_not_match
}

pub use super::import::relationship_from_json as from_json;