use serde::*;
use super::{*, mod_version::ModVersion};

/// Describes a module using an identifier and version requirement.
/// 
/// # Usage
/// It is an error to use `version` with either `min_version` or `max_version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDescriptor {
	pub name: String,
	/* TODO: Use enum to enforce usage restrictions */
	pub version: Option<ModVersion>,
	pub min_version: Option<ModVersion>,
	pub max_version: Option<ModVersion>,
} 

impl ModuleDescriptor {
	/// It is an error to use `version` with either `min_version` or `max_version`
	pub fn new(name: String, version: Option<String>, min_version: Option<String>, max_version: Option<String> ) -> Self {
		/* TODO: Don't panic */
		if version.is_some() && (min_version.is_some() || max_version.is_some()) { panic!("relationship entry can't mix version with min_version or max_version") }
		
		Self {
			name,
			version: version.map(|v| ModVersion::new(&v).unwrap()),
			min_version: min_version.map(|v| ModVersion::new(&v).unwrap()),
			max_version: max_version.map(|v| ModVersion::new(&v).unwrap()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relationship {
	AnyOf(Vec<ModuleDescriptor>),
	One(ModuleDescriptor),
}

impl Relationship {
	/// Convienience function to collapse this relationship into a vector
	pub fn as_vec(&self) -> Vec<&ModuleDescriptor> {
		match self {
			Relationship::AnyOf(v) => v.iter().collect::<Vec<_>>(),
			Relationship::One(r) => vec![r],
		}
	}
}

pub fn does_module_fulfill_relationship(module: &Ckan, relationship: &Relationship) -> bool {
	for desc in relationship.as_vec() {
		if does_module_match_descriptor(module, desc) { return true }
	}
	false
}

pub fn does_module_match_descriptor(module: &Ckan, descriptor: &ModuleDescriptor) -> bool {
	if module.identifier != descriptor.name && !module.provides.iter().any(|m| m == &descriptor.name) {
		return false
	}
	match (&descriptor.version, &descriptor.min_version, &descriptor.max_version) {
		(None, None, None) => true,
		(None, None, Some(max)) => &module.version <= max,
		(None, Some(min), None) => &module.version >= min,
		(None, Some(min), Some(max)) => min <= &module.version && &module.version <= max,
		(Some(v), None, None) => &module.version == v,
		_ => panic!("invalid relationship entry")
	}
}

pub use super::import::relationship_from_json as from_json;