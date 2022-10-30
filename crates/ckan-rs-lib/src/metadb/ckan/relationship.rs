use serde::*;
use super::{*, mod_version::ModVersion};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModUniqueIdentifier {
	pub identifier: String,
	pub version: ModVersion,
}

impl std::cmp::Ord for ModUniqueIdentifier {
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

impl std::cmp::PartialOrd for ModUniqueIdentifier {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl std::fmt::Display for ModUniqueIdentifier {
	fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}-{}", self.identifier, self.version)
	}
}

pub type ModVersionBounds = VersionBounds<ModVersion>;

/// Describes a module using an identifier and version requirement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleDescriptor {
	pub name: String,
	pub version: ModVersionBounds,
} 

impl ModuleDescriptor {
	/// It is an error to use `version` with either `min_version` or `max_version`
	pub fn new(name: String, version: ModVersionBounds) -> Self {
		Self {
			name,
			version,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

pub fn does_module_fulfill_relationship(module: &ModuleInfo, relationship: &Relationship) -> bool {
	for desc in relationship.as_vec() {
		if does_module_provide_descriptor(module, desc) { return true }
	}
	false
}

pub fn does_module_match_descriptor(identifier: &ModUniqueIdentifier, descriptor: &ModuleDescriptor) -> bool {
	if identifier.identifier != descriptor.name {
		return false
	}
	descriptor.version.is_version_within(&identifier.version)
}

pub fn does_module_provide_descriptor(module: &ModuleInfo, descriptor: &ModuleDescriptor) -> bool {
	if module.unique_id.identifier != descriptor.name && !module.provides.iter().any(|m| m == &descriptor.name) {
		return false
	}
	descriptor.version.is_version_within(&module.unique_id.version)
}

pub use super::import::relationship_from_json as from_json;