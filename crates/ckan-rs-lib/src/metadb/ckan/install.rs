use serde::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceDirective {
	File(String),
	Find(String),
	FindRegExp(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionalDirective {
	As(String),
	Filter(Vec<String>),
	FilterRegExp(Vec<String>),
	IncludeOnly(Vec<String>),
	IncludeOnlyRegExp(Vec<String>),
	FindMatchesFiles(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallDirective {
	pub source: SourceDirective,
	pub install_to: String,
	pub additional: Vec<OptionalDirective>,
} impl InstallDirective {
	pub fn new(source: SourceDirective, install_to: String, additional: Vec<OptionalDirective>) -> Self {
		Self { source, install_to, additional }
	}
}