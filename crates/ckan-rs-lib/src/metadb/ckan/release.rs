use serde::*;

#[derive(Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReleaseStatus {
	#[default] Stable,
	Testing,
	Development,
}