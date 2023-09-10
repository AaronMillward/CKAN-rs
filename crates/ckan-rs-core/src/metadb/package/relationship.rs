use serde::*;
use super::*;

/// A unique identifier for packages.
/// 
/// Mainly used as an index into [`crate::MetaDB`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PackageIdentifier {
	pub identifier: String,
	pub version: PackageVersion,
}

impl std::cmp::Ord for PackageIdentifier {
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

impl std::cmp::PartialOrd for PackageIdentifier {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl std::fmt::Display for PackageIdentifier {
	fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}-{}", self.identifier, self.version)
	}
}

impl AsRef<PackageIdentifier> for PackageIdentifier {
	fn as_ref(&self) -> &PackageIdentifier {
		self
	}
}

/// Describes a package using an identifier and version requirement.
/// 
/// Differs from [`PackageIdentifier`] in that it represents a range of packages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageDescriptor {
	pub name: String,
	pub version: PackageVersionBounds,
} 

impl PackageDescriptor {
	/// It is an error to use `version` with either `min_version` or `max_version`
	pub fn new(name: String, version: PackageVersionBounds) -> Self {
		Self {
			name,
			version,
		}
	}
}

/// A requirement of a package that must be met for the package to be installed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Relationship {
	/// At least one of the descriptors must match to fulfill the relationship.
	AnyOf(Vec<PackageDescriptor>),
	/// This single descriptor requirement must be met.
	One(PackageDescriptor),
}

impl Relationship {
	/* TODO: Iterator */
	/// Convienience function to collapse this relationship into a vector
	pub fn as_vec(&self) -> Vec<&PackageDescriptor> {
		match self {
			Relationship::AnyOf(v) => v.iter().collect::<Vec<_>>(),
			Relationship::One(r) => vec![r],
		}
	}
}

pub fn does_package_fulfill_relationship(package: &Package, relationship: &Relationship) -> bool {
	for desc in relationship.as_vec() {
		if does_package_provide_descriptor(package, desc) { return true }
	}
	false
}

pub fn does_package_match_descriptor(identifier: &PackageIdentifier, descriptor: &PackageDescriptor) -> bool {
	if identifier.identifier != descriptor.name {
		return false
	}
	descriptor.version.is_version_within(&identifier.version)
}

pub fn does_package_provide_descriptor(package: &Package, descriptor: &PackageDescriptor) -> bool {
	if package.identifier.identifier != descriptor.name && !package.provides.iter().any(|m| m == &descriptor.name) {
		return false
	}
	descriptor.version.is_version_within(&package.identifier.version)
}