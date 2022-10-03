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
		self.mod_version == other.mod_version
	}
}

impl PartialOrd for ModVersion {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		if self.epoch != other.epoch {
			self.epoch.partial_cmp(&other.epoch)
		} else {
			/* TODO:FIXME: the spec is very wordy about how this is compaired, just doing something basic for now */
			let mut ord = std::cmp::Ordering::Equal;
			for (lhs,rhs) in self.mod_version.chars().zip(other.mod_version.chars()) {
				let res = lhs.partial_cmp(&rhs).unwrap();
				match res {
					std::cmp::Ordering::Equal => continue,
					_ => ord = res,
				}
			}
			Some(ord)
		}
	}
}

impl std::hash::Hash for ModVersion {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.epoch.hash(state);
		self.mod_version.hash(state);
	}
}