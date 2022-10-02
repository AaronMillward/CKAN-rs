use serde::*;

#[derive(Debug, Eq, Serialize, Deserialize)]
pub struct KspVersion {
	build: i32,
	name: String,
}

/* TODO: Version comparison functions. PartialEq doesn't quite cover the possible cases */

impl PartialEq for KspVersion {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl PartialOrd for KspVersion {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		let mut ord = std::cmp::Ordering::Equal;
		for (lhs,rhs) in self.name.chars().zip(other.name.chars()) {
			let res = lhs.partial_cmp(&rhs).unwrap();
			match res {
				std::cmp::Ordering::Equal => continue,
				_ => ord = res,
			}
		}
		Some(ord)
	}
}

impl std::hash::Hash for KspVersion {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}