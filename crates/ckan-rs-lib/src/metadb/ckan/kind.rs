use serde::*;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
	/// A normal installable module.
	#[default] Package,
	/// A distributable .ckan file that has relationships to other mods while having no download of its own.
	MetaPackage,
	/// A paid expansion from SQUAD, which CKAN can detect but not install. Also has no download.
	DLC,
}