use serde::*;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseStatus {
	Stable,
	Testing,
	Development,
}
impl Default for ReleaseStatus { fn default() -> Self { Self::Stable } }