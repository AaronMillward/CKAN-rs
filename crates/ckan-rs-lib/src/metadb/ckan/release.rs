use serde::*;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReleaseStatus {
	Stable,
	Testing,
	Development,
}
impl Default for ReleaseStatus { fn default() -> Self { Self::Stable } }