use serde::*;

#[derive(Debug, Eq, Serialize, Deserialize)]
pub struct KspVersion {
	name: String,
}

impl KspVersion {
	/// Checks if given versions are compatible with each other
	pub fn is_compatible(lhs: &KspVersion, rhs: &KspVersion) -> bool {
		if lhs.name == "any" || rhs.name == "any" { return true }
		if lhs.name.starts_with(&rhs.name) { return true }
		if rhs.name.starts_with(&lhs.name) { return true }
		false
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

impl PartialEq for KspVersion {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl PartialOrd for KspVersion {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some({
			fn get_string_until_numeric(s: &str) -> (&str,&str) {
				let mut split = 0;
				for (i,c) in s.chars().enumerate() {
					if c.is_numeric() {
						split = i;
						break;
					}
				}
				if split == 0 {
					return (s, "")
				}
				s.split_at(split)
			}

			fn get_string_until_not_numeric(s: &str) -> (&str,&str) {
				let mut split = 0;
				for (i,c) in s.chars().enumerate() {
					if !c.is_numeric() {
						split = i;
						break;
					}
				}
				if split == 0 {
					return (s, "")
				}
				s.split_at(split.max(0))
			}

			match (self.name == "any", other.name == "any") {
				(true, true) => return Some(std::cmp::Ordering::Equal),
				(true, false) => return Some(std::cmp::Ordering::Greater),
				(false, true) => return Some(std::cmp::Ordering::Less),
				(false, false) => {},
			}

			let mut lhs: (&str, &str) = ("", &self.name);
			let mut rhs: (&str, &str) = ("", &other.name);
			
			while !lhs.1.is_empty() && !rhs.1.is_empty() {
				lhs = get_string_until_numeric(lhs.1);
				rhs = get_string_until_numeric(rhs.1);

				match lhs.0.cmp(rhs.0) {
					std::cmp::Ordering::Equal => {},
					ord => return Some(ord)
				}

				lhs = get_string_until_not_numeric(lhs.1);
				rhs = get_string_until_not_numeric(rhs.1);

				if !lhs.0.is_empty() && !rhs.0.is_empty() {
					let lhs_num = lhs.0.parse::<i32>().expect("can't parse version number as int");
					let rhs_num = rhs.0.parse::<i32>().expect("can't parse version number as int");
					
					match lhs_num.cmp(&rhs_num) {
						std::cmp::Ordering::Equal => {},
						ord => return Some(ord)
					}
				}
			}

			lhs.1.len().cmp(&rhs.1.len())
		})
	}
}

impl std::hash::Hash for KspVersion {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}