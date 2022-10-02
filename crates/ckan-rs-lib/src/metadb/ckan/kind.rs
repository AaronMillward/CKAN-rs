use serde::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
	Package,
	MetaPackage,
	Dlc,
}
impl Default for Kind { fn default() -> Self { Self::Package } }