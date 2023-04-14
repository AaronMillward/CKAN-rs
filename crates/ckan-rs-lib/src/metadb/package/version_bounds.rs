use serde::{Serialize, Deserialize};

/// A generic enum to describe a range of versions.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum VersionBounds<T>
where T: std::cmp::PartialEq + std::cmp::Ord + std::clone::Clone,
{
	#[default] Any,
	Explicit(T),
	MinOnly(T),
	MaxOnly(T),
	MinMax(T, T),
}

impl<T> VersionBounds<T>
where T: std::cmp::PartialEq + std::cmp::Ord + std::clone::Clone,
{
	/// When all arguments are `None` will return `Any`
	pub fn new(explicit: Option<T>, min: Option<T>, max: Option<T>) -> crate::Result<VersionBounds<T>> {
		match (explicit, min, max) {
			(None, None, None) => Ok(VersionBounds::Any),
			(None, None, Some(max)) => Ok(VersionBounds::MaxOnly(max)),
			(None, Some(min), None) => Ok(VersionBounds::MinOnly(min)),
			(None, Some(min), Some(max)) => Ok(VersionBounds::MinMax(min, max)),
			(Some(e), None, None) => Ok(VersionBounds::Explicit(e)),
			_ => Err(crate::Error::Parse("Attempted to create bounds with both explicit and min or max version constraint".to_string()))
		}
	}

	pub fn is_version_within(&self, other: &T) -> bool {
		match self {
			VersionBounds::Any => true,
			VersionBounds::Explicit(v) => other == v,
			VersionBounds::MinOnly(min) => other >= min,
			VersionBounds::MaxOnly(max) => other <= max,
			VersionBounds::MinMax(min, max) => min <= other && other <= max,
		}
	}

	/// Gets the intersection between the bounds, if no intersection exists returns `None`
	pub fn inner_join(&self, other: &Self) -> Option<Self> {
		let lhs = self.clone();
		let rhs = other.clone();

		match (lhs, rhs) {
			(VersionBounds::Any, r) => Some(r),
			(l, VersionBounds::Any) => Some(l),
			
			(VersionBounds::Explicit(a), VersionBounds::Explicit(b)) => if a == b { Some(VersionBounds::Explicit(a)) } else { None },
			(VersionBounds::Explicit(a), b) => if b.is_version_within(&a) { Some(VersionBounds::Explicit(a)) } else { None },
			(a, VersionBounds::Explicit(b)) => if a.is_version_within(&b) { Some(VersionBounds::Explicit(b)) } else { None },

			(VersionBounds::MinOnly(a), VersionBounds::MinOnly(b)) => Some(VersionBounds::MinOnly(std::cmp::max(a,b))),
			(VersionBounds::MaxOnly(a), VersionBounds::MaxOnly(b)) => Some(VersionBounds::MaxOnly(std::cmp::min(a,b))),
			
			(VersionBounds::MinOnly(a), VersionBounds::MaxOnly(b)) | (VersionBounds::MaxOnly(b), VersionBounds::MinOnly(a)) => if a < b { Some(VersionBounds::MinMax(a,b)) } else { None },

			(VersionBounds::MinOnly(a), VersionBounds::MinMax(b, c)) | (VersionBounds::MinMax(b, c), VersionBounds::MinOnly(a)) => {
				let min = std::cmp::max(a,b);
				if min > c {
					None
				} else {
					Some(VersionBounds::MinMax(min, c))
				}
			}

			(VersionBounds::MaxOnly(a), VersionBounds::MinMax(b, c)) | (VersionBounds::MinMax(b, c), VersionBounds::MaxOnly(a)) => {
				let max = std::cmp::min(a.clone(), c);
				if max < a {
					None
				} else {
					Some(VersionBounds::MinMax(b, max))
				}
			}

			(VersionBounds::MinMax(a, b), VersionBounds::MinMax(c, d)) => {
				let min = std::cmp::max(a,c);
				let max = std::cmp::min(b,d);
				if min < max { Some(VersionBounds::MinMax(min, max)) } else { None }
			},
		}
	}
}
