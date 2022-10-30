use serde::*;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct ModVersion {
	epoch: i32,
	mod_version: String,
}

impl ModVersion {
	pub fn new(version: &str) -> crate::Result<Self> {
		/* FIXME: mod_version can be *any* string so this method assumes mod_version doesn't contain a ':' */
		let spl: Vec<&str> = version.splitn(2,':').collect();
		Ok(ModVersion {
			epoch: {
				spl[0].parse::<i32>().unwrap_or(0) /* FIXME: We assume spl[0] is not just a number */
			},
			mod_version: {
				spl[spl.len() - 1].to_string()
			}
		})
	}
}

impl TryFrom<String> for ModVersion {
	type Error = crate::Error;
	fn try_from(value: String) -> Result<Self, Self::Error> { Self::new(&value) }
}

impl PartialEq for ModVersion {
	fn eq(&self, other: &Self) -> bool {
		self.epoch == other.epoch &&
		self.mod_version == other.mod_version
	}
}

impl Ord for ModVersion {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
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

				let mut lhs: (&str, &str) = ("", &self.mod_version);
				let mut rhs: (&str, &str) = ("", &other.mod_version);
				
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

impl PartialOrd for ModVersion {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl std::hash::Hash for ModVersion {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.epoch.hash(state);
		self.mod_version.hash(state);
	}
}

impl std::fmt::Display for ModVersion {
	fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.epoch, self.mod_version)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test] fn mod_version_are_not_compared_lexically() { assert!(ModVersion::new("1.2.4.0").unwrap() < ModVersion::new("1.2.10.0").unwrap()) }
	#[test] fn mod_version_short_version_is_lt() { assert!(ModVersion::new("1.2").unwrap() < ModVersion::new("1.2.3").unwrap()) }
	#[test] fn mod_version_identical_are_eq() { assert!(ModVersion::new("1.2.3").unwrap() == ModVersion::new("1.2.3").unwrap()) }
	#[test] fn mod_version_higher_version_is_gt() { assert!(ModVersion::new("1.2.3").unwrap() < ModVersion::new("1.2.4").unwrap()) }
	#[test] fn mod_version_prefix_is_supported() { assert!(ModVersion::new("v1.2.3").unwrap() < ModVersion::new("v1.2.4").unwrap()) }
	#[test] fn mod_version_prefix_is_compared_lexically() { assert!(ModVersion::new("a1.2.3").unwrap() < ModVersion::new("b1.2.3").unwrap()) }
	#[test] fn mod_version_trailing_non_digit() { assert!(ModVersion::new("1.2a").unwrap() < ModVersion::new("1.2b").unwrap()) }
	#[test] fn mod_version_trailing_digit() { assert!(ModVersion::new("1.2").unwrap() < ModVersion::new("1.3").unwrap()) }
	#[test] fn mod_version_epoch_is_respected() { assert!(ModVersion::new("1:1.2").unwrap() < ModVersion::new("2:v0.1").unwrap()) }
}