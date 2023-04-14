//! Structs representing KSP version numbers.
//! 
//! # KSP Versioning Numbers
//! 
//! Although we use the terms major/minor/patch for the components KSP does not use semantic versioning
//! where instead breaking changes usually occur on minor version bumps.
//! 
//! # "Generally Compatible"
//! 
//! According to the CKAN Specification it is up to clients to determine what versions are considered generally compatible.
//! Check the documentation for [`KspVersionReal::is_compatible_with()`] for a breakdown of what is compatible.

/* XXX:
	I have some uncertainty in how to implement the "any" version, I believe the current implementation
	is okay but may have unforseen issues.
	
	The spec implies the fields ksp_version, ksp_version_min and ksp_version_max can be "any", this raises
	a problem as VersionBounds<T> requires T: Ord but the presence of an "any" variant would mean only
	PartialOrd could be implemented.

	All these conditions mean that representing the bounds as show in the spec is difficult to implement. but it
	seems as though we may not need to accurately represent the data.

	It's kind of hacky but min/max being "any" is the same as it not being present so we don't need to represent this state and
	an explicit "any" can be represented by the VersionBounds::Any variant. this also works because currently we have no need
	for an any state outside of VersionBounds.
 */

use serde::*;
use try_map::FallibleMapExt;

/// Represents a specific KSP Version.
/// 
/// # Format
/// 
/// KSP version numbers follow the format:
/// 
/// `MAJOR`.`MINOR`.`PATCH`.`BUILD`
/// 
/// For example: `1.12.3.3173`
/// 
/// `MAJOR` and `MINOR` are required whereas `PATCH` and `BUILD` are optional.
/// 
/// # What This Does Not Represent
/// 
/// The "any" version description. Use [`KspVersionBounds`] for this.
/// 
/// # Eq & Ord
/// 
/// The `build` number is not considered in Eq and Ord as it causes problems
/// in cases such as `1.12.3 < 1.12.3.3173` which come up when attempting to compare
/// an instance version with a mod version constraint.
/// 
/// The `Ord` implementation should not be used for checking compatibility. for example
/// some packages may claim compatibility with `1.12` implying all patches however an instance of version `1.12.3`
/// would be considered higher by the Ord implementation.
/// 
/// Instead use [`KspVersionReal::is_compatible_with()`] or [`KspVersionBounds::is_version_compatible()`] to check compatibility.
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct KspVersionReal {
	major: u32,
	minor: u32,
	patch: Option<u32>,
	build: Option<u32>,
}

impl KspVersionReal {
	/// Create a new [`KspVersionReal`] from a version string.
	/// 
	/// # Errors
	/// This function will return a [`Parse`](crate::Error::Parse) error in the following cases.
	/// - Input is an "any" string.
	/// - Input doesn't include a minor version.
	/// - Input has more components than the `MAJOR`.`MINOR`.`PATCH`.`BUILD` format.
	/// - The components of the version can't be parsed as integers.
	pub fn new(s: impl AsRef<str>) -> crate::Result<Self> {
		use crate::Error::Parse;
		let s = s.as_ref();
		if s.to_lowercase() == "any" { return Err(Parse("\"any\" is not a real version".into())) }
		let components = s.split('.').collect::<Vec<_>>();
		if components.len() < 2 || components.len() > 4 { return Err(Parse("too many/few version components".into())) }
		
		#[allow(clippy::get_first)]
		let major = components.get(0).expect("len() should be confirmed >2.").parse::<u32>().map_err(|_| Parse("Major version can't be parsed".into()))?;
		let minor = components.get(1).expect("len() should be confirmed >2.").parse::<u32>().map_err(|_| Parse("Minor version can't be parsed".into()))?;
		let patch = components.get(2).try_map(|v| v.parse::<u32>().map_err(|_| Parse("patch can't be parsed".into())))?;
		let build = components.get(3).try_map(|v| v.parse::<u32>().map_err(|_| Parse("build can't be parsed".into())))?;
		
		Ok(KspVersionReal { major, minor, patch, build, })
	}

	/// Checks general compatibility between two versions.
	/// 
	/// # How This Is Defined
	/// 1. `major` and `minor` must match.
	/// 1. If `patch` is present in both, `self <= other`.
	/// any other combination of patch options is considered compatible.
	pub fn is_compatible_with(&self, other: &Self) -> bool {
		if self.major() == other.major() && self.minor() == other.minor() {
			match (self.patch(), other.patch()) {
				(Some(lhs), Some(rhs)) => lhs <= rhs,
				_ => true,
			}
		} else {
			false
		}
		/* TODO: Compatibility Matrix */
	}

	pub fn major(&self) -> u32 { self.major }
	pub fn set_major(&mut self, major: u32) { self.major = major; }
	pub fn minor(&self) -> u32 { self.minor }
	pub fn set_minor(&mut self, minor: u32) { self.minor = minor; }
	pub fn patch(&self) -> Option<u32> { self.patch }
	pub fn set_patch(&mut self, patch: Option<u32>) { self.patch = patch; }
	pub fn build(&self) -> Option<u32> { self.build }
	pub fn set_build(&mut self, build: Option<u32>) { self.build = build; }
}

impl TryFrom<&str> for KspVersionReal {
	type Error = crate::Error;
	
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl PartialEq for KspVersionReal {
	fn eq(&self, other: &Self) -> bool {
		self.major() == other.major() &&
		self.minor() == other.minor() &&
		self.patch() == other.patch()
	}
}

impl Ord for KspVersionReal {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		/* XXX:
			We don't consider the build field when ordering as it complicates a number of processes
			notebly in cases such as `1.12.3 < 1.12.3.3173` which come up when attempting to compare
			an instance version with a mod version constraint.
		 */
		let lhs = vec![Some(self.major()), Some(self.minor()), self.patch()];
		let rhs = vec![Some(other.major()), Some(other.minor()), other.patch()];
	
		for (lhs, rhs) in lhs.into_iter().zip(rhs) {
			match lhs.cmp(&rhs) {
				std::cmp::Ordering::Equal => { continue; },
				c => return c,
			}
		}

		std::cmp::Ordering::Equal
	}
}

impl PartialOrd for KspVersionReal {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

pub type KspVersionBounds = super::VersionBounds<KspVersionReal>;

impl KspVersionBounds {
	/// Generate a new version bounds from a version string such as `"1.12.3"`
	/// 
	/// # Errors
	/// Only generates the same errors as [`KspVersionReal::new()`] when creating versions from the input.
	pub fn new_from_str(explicit: Option<impl AsRef<str>>, min: Option<impl AsRef<str>>, max: Option<impl AsRef<str>>) -> crate::Result<Self> {
		if let Some(ref explicit) = explicit {
			if explicit.as_ref().to_lowercase() == "any" {
				return Ok(Self::Any)
			}
		}

		let min = min.and_then(|s| {
			if s.as_ref().to_lowercase() == "any" { None }
			else { Some(s) }
		});

		let max = max.and_then(|s| {
			if s.as_ref().to_lowercase() == "any" { None }
			else { Some(s) }
		});

		let explicit = explicit.try_map(|s| KspVersionReal::new(s))?;
		let min = min.try_map(|s| KspVersionReal::new(s))?;
		let max = max.try_map(|s| KspVersionReal::new(s))?;

		super::VersionBounds::new(explicit, min, max)
	}

	/// Checks if `other` is a version compatible with this version bound.
	/// 
	/// # Parameters
	/// - `strict` - Should this check require the version to be exactly equal or just compatible.
	pub fn is_version_compatible(&self, other: &KspVersionReal, strict: bool) -> bool {
		match self {
			Self::Any => true,
			Self::Explicit(v) => {
				if strict {
					other == v
				} else {
					v.is_compatible_with(other)
				}
			},
			Self::MinOnly(min) => other >= min || other.is_compatible_with(min),
			Self::MaxOnly(max) => other <= max || other.is_compatible_with(max),
			Self::MinMax(min, max) => (min <= other || other.is_compatible_with(min)) && (other <= max || other.is_compatible_with(max)),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test] fn ksp_version_compares_as_ints() { assert!(KspVersionReal::new("1.9").unwrap() < KspVersionReal::new("1.10").unwrap()) }
	#[test] fn ksp_version_short_version_is_lt() { assert!(KspVersionReal::new("1.12").unwrap() < KspVersionReal::new("1.12.1").unwrap()) }
	#[test] fn ksp_version_short_version_is_gt_smaller_long_version() { assert!(KspVersionReal::new("1.11.1").unwrap() < KspVersionReal::new("1.12").unwrap()) }
	#[test] fn ksp_version_identical_are_eq() { assert!(KspVersionReal::new("1.12.1").unwrap() == KspVersionReal::new("1.12.1").unwrap()) }
	#[test] fn ksp_version_higher_version_is_gt() { assert!(KspVersionReal::new("1.12.1").unwrap() < KspVersionReal::new("1.12.2").unwrap()) }
	#[test] fn ksp_version_le() { assert!(KspVersionReal::new("1.10").unwrap() <= KspVersionReal::new("1.11").unwrap() && KspVersionReal::new("1.10").unwrap() <= KspVersionReal::new("1.10").unwrap()) }
	#[test] fn ksp_version_ge() { assert!(KspVersionReal::new("1.11").unwrap() <= KspVersionReal::new("1.12").unwrap() && KspVersionReal::new("1.11").unwrap() <= KspVersionReal::new("1.11").unwrap()) }
	#[test] fn ksp_version_build_has_no_effect() { assert!(KspVersionReal::new("1.12.1").unwrap() == KspVersionReal::new("1.12.1.1234").unwrap()) }
}