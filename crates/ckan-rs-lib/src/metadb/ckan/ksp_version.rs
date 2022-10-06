use serde::*;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct KspVersion {
	/* TODO: store version numbers as ints here to avoid string processing in comparison */
	// any: bool,
	// major: Option<u32>,
	// minor: Option<u32>,
	// fix: Option<u32>,
	name: String,
}

impl KspVersion {
	/// Checks if `rhs` is a sub version of `lhs` 
	/// 
	/// if either `lhs` or `rhs` are "any" returns false
	/// 
	/// # Examples
	/// ```
	/// assert!(is_sub_version(&KspVersion::new("1.12"), &KspVersion::new("1.12.2"))
	/// ```
	pub fn is_sub_version(lhs: &KspVersion, rhs: &KspVersion) -> bool {
		if lhs.name == "any" || rhs.name == "any" { return false }
		if rhs.name.starts_with(&lhs.name) { return true }
		false
	}

	pub fn is_any(&self) -> bool {
		self.name == "any"
	}
}

impl KspVersion {
	pub fn new(s: &str) -> Self {
		Self { name: s.to_string() }
	}
}

impl From<&str> for KspVersion {
	fn from(s: &str) -> Self {
		Self::new(s)
	}
}

impl Ord for KspVersion {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		match (self.name == "any", other.name == "any") {
			(true, true) => return std::cmp::Ordering::Equal,
			(true, false) => return std::cmp::Ordering::Greater,
			(false, true) => return std::cmp::Ordering::Less,
			(false, false) => {},
		}

		let lhs = self.name.split('.').collect::<Vec<_>>();
		let rhs = other.name.split('.').collect::<Vec<_>>();

		for (lhs, rhs) in lhs.iter().zip(rhs.iter()) {
			let lhs_num = lhs.parse::<i32>().expect("version isn't a number");
			let rhs_num = rhs.parse::<i32>().expect("version isn't a number");
			match lhs_num.cmp(&rhs_num) {
				std::cmp::Ordering::Equal => {},
				ord => return ord,
			}
		}
		
		lhs.len().cmp(&rhs.len())
	}
}

impl PartialOrd for KspVersion {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for KspVersion {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl std::hash::Hash for KspVersion {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test] fn ksp_version_compares_as_ints() { assert!(KspVersion::new("1.9") < KspVersion::new("1.10")) }
	#[test] fn ksp_version_short_version_is_lt() { assert!(KspVersion::new("1.12") < KspVersion::new("1.12.1")) }
	#[test] fn ksp_version_short_version_is_gt_smaller_long_version() { assert!(KspVersion::new("1.11.1") < KspVersion::new("1.12")) }
	#[test] fn ksp_version_identical_are_eq() { assert!(KspVersion::new("1.12.1") == KspVersion::new("1.12.1")) }
	#[test] fn ksp_version_higher_version_is_gt() { assert!(KspVersion::new("1.12.1") < KspVersion::new("1.12.2")) }
	#[test] fn ksp_version_is_sub_version() { assert!(KspVersion::is_sub_version(&KspVersion::new("1.12"), &KspVersion::new("1.12.2"))) }
	#[test] fn ksp_version_is_not_sub_version() { assert!(!KspVersion::is_sub_version(&KspVersion::new("1.11"), &KspVersion::new("1.12.2"))) }
	#[test] fn ksp_version_le() { assert!(KspVersion::new("1.10") <= KspVersion::new("1.11") && KspVersion::new("1.10") <= KspVersion::new("1.10")) }
	#[test] fn ksp_version_ge() { assert!(KspVersion::new("1.11") <= KspVersion::new("1.12") && KspVersion::new("1.11") <= KspVersion::new("1.11")) }
}