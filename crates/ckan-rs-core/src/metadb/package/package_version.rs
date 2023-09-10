use serde::*;

/// The version of a package.
/// 
/// # Format
/// Package versions follow a format `[epoch:]mod_version`.
/// - `epoch` is used to correct errors in versioning schemes or
/// organize versions that are difficult to interpret.
/// - `version` can technically be *any* string according to the spec.
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct PackageVersion {
	epoch: i32,
	version: String,
}

impl PackageVersion {
	/* TODO: impl AsRef<str> */
	pub fn new(version: &str) -> crate::Result<Self> {
		/* FIXME: version can be *any* string so this method assumes version doesn't contain a ':' */
		let spl: Vec<&str> = version.splitn(2,':').collect();
		Ok(PackageVersion {
			epoch: {
				spl[0].parse::<i32>().unwrap_or(0) /* FIXME: We assume spl[0] is not just a number */
			},
			version: {
				spl[spl.len() - 1].to_string()
			}
		})
	}
}

impl TryFrom<String> for PackageVersion {
	type Error = crate::Error;
	fn try_from(value: String) -> Result<Self, Self::Error> { Self::new(&value) }
}

impl PartialEq for PackageVersion {
	fn eq(&self, other: &Self) -> bool {
		self.epoch == other.epoch &&
		self.version == other.version
	}
}

impl Ord for PackageVersion {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		/* NOTE:
			Here's an extract from the spec explaining this mess.
			
			"
			When comparing two version numbers, first the epoch of each are compared, then the mod_version if epoch is equal. epoch is compared numerically. The mod_version part is compared by the package management system using the following algorithm:

			The strings are compared from left to right.

			First the initial part of each string consisting entirely of non-digit characters is determined. These two parts (one of which may be empty) are compared lexically. If a difference is found it is returned. The lexical comparison is a comparison of ASCII values modified so that all the letters sort earlier than all the non-letters.

			Then the initial part of the remainder of each string which consists entirely of digit characters is determined. The numerical values of these two parts are compared, and any difference found is returned as the result of the comparison. For these purposes an empty string (which can only occur at the end of one or both version strings being compared) counts as zero.

			These two steps (comparing and removing initial non-digit strings and initial digit strings) are repeated until a difference is found or both strings are exhausted.

			Note that the purpose of epochs is to allow us to leave behind mistakes in version numbering, and to cope with situations where the version numbering scheme changes. It is not intended to cope with version numbers containing strings of letters which the package management system cannot interpret (such as ALPHA or pre-), or with silly orderings.
			"
		 */

		match self.epoch.cmp(&other.epoch) {
			std::cmp::Ordering::Equal => {
				fn get_string_until_numeric(s: &str) -> (&str,&str) {
					if s.is_empty() {
						return ("", "")
					}

					/* Without this versions starting with a numeric return the entire string at once.
					 * If that happens it's hard to spot because the string will still compare 
					 * lexically and work for most cases.
					*/
					if let Some(c) = s.chars().next() {
						if c.is_numeric() {
							return ("", s)
						}
					}

					if s.len() == 1 {
						return (s, "")
					}

					let mut split: Option<usize> = None;
					for (i,c) in s.chars().enumerate() {
						if c.is_numeric() {
							split = Some(i);
							break;
						}
					}
					if let Some(i) = split {
						s.split_at(i)
					} else {
						(s,"")
					}
				}

				fn get_string_until_not_numeric(s: &str) -> (&str,&str) {
					if s.is_empty() {
						return ("", "")
					}

					if let Some(c) = s.chars().next() {
						if !c.is_numeric() {
							return ("", s)
						}
					}

					if s.len() == 1 {
						return (s, "")
					}

					let mut split: Option<usize> = None;
					for (i,c) in s.chars().enumerate() {
						if !c.is_numeric() {
							split = Some(i);
							break;
						}
					}
					if let Some(i) = split {
						s.split_at(i)
					} else {
						(s,"")
					}
				}

				let mut lhs: (&str, &str) = ("", &self.version);
				let mut rhs: (&str, &str) = ("", &other.version);
				
				while !lhs.1.is_empty() && !rhs.1.is_empty() {
					lhs = get_string_until_numeric(lhs.1);
					rhs = get_string_until_numeric(rhs.1);

					match lhs.0.cmp(rhs.0) {
						std::cmp::Ordering::Equal => {},
						ord => return ord
					}

					lhs = get_string_until_not_numeric(lhs.1);
					rhs = get_string_until_not_numeric(rhs.1);

					if !lhs.0.is_empty() && !rhs.0.is_empty() {
						let lhs_num = lhs.0.parse::<i32>().expect("can't parse version number as int");
						let rhs_num = rhs.0.parse::<i32>().expect("can't parse version number as int");
						
						match lhs_num.cmp(&rhs_num) {
							std::cmp::Ordering::Equal => {},
							ord => return ord
						}
					}
				}

				lhs.1.len().cmp(&rhs.1.len())
			},
			ord => ord,
		}
	}
}

impl PartialOrd for PackageVersion {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl std::hash::Hash for PackageVersion {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.epoch.hash(state);
		self.version.hash(state);
	}
}

impl std::fmt::Display for PackageVersion {
	fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.epoch, self.version)
	}
}

pub type PackageVersionBounds = crate::metadb::package::version_bounds::VersionBounds<PackageVersion>;

#[cfg(test)]
mod test {
	use super::*;

	#[test] fn mod_version_are_not_compared_lexically() { assert!(PackageVersion::new("1.2.4.0").unwrap() < PackageVersion::new("1.2.10.0").unwrap()) }
	#[test] fn mod_version_short_version_is_lt() { assert!(PackageVersion::new("1.2").unwrap() < PackageVersion::new("1.2.3").unwrap()) }
	#[test] fn mod_version_identical_are_eq() { assert!(PackageVersion::new("1.2.3").unwrap() == PackageVersion::new("1.2.3").unwrap()) }
	#[test] fn mod_version_higher_version_is_gt() { assert!(PackageVersion::new("1.2.3").unwrap() < PackageVersion::new("1.2.4").unwrap()) }
	#[test] fn mod_version_prefix_is_supported() { assert!(PackageVersion::new("v1.2.3").unwrap() < PackageVersion::new("v1.2.4").unwrap()) }
	#[test] fn mod_version_prefix_is_compared_lexically() { assert!(PackageVersion::new("a1.2.3").unwrap() < PackageVersion::new("b1.2.3").unwrap()) }
	#[test] fn mod_version_trailing_non_digit() { assert!(PackageVersion::new("1.2a").unwrap() < PackageVersion::new("1.2b").unwrap()) }
	#[test] fn mod_version_trailing_digit() { assert!(PackageVersion::new("1.2").unwrap() < PackageVersion::new("1.3").unwrap()) }
	#[test] fn mod_version_epoch_is_respected() { assert!(PackageVersion::new("1:1.2").unwrap() < PackageVersion::new("2:v0.1").unwrap()) }
}