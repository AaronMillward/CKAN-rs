use serde::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
	Package,
	MetaPackage,
	Dlc,
}
impl Default for Kind { fn default() -> Self { Self::Package } }