use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VersionBounds<T>
where T: std::cmp::PartialEq + std::cmp::PartialOrd,
{
	Any,
	Explicit(T),
	MinOnly(T),
	MaxOnly(T),
	MinMax(T, T),
}

impl<T> VersionBounds<T>
where T: std::cmp::PartialEq + std::cmp::PartialOrd,
{
	/// When all arguments are `None` will return `Any`
	pub fn new(explicit: Option<T>, min: Option<T>, max: Option<T>) -> crate::Result<VersionBounds<T>> {
		match (explicit, min, max) {
			(None, None, None) => Ok(VersionBounds::Any),
			(None, None, Some(max)) => Ok(VersionBounds::MaxOnly(max)),
			(None, Some(min), None) => Ok(VersionBounds::MinOnly(min)),
			(None, Some(min), Some(max)) => Ok(VersionBounds::MinMax(min, max)),
			(Some(e), None, None) => Ok(VersionBounds::Explicit(e)),
			_ => Err(crate::Error::ParseError("using both version and min or max version constraint".to_string()))
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
}
