use serde::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourceDirective {
	#[serde(rename = "file")]
	File(String),
	#[serde(rename = "find")]
	Find(String),
	#[serde(rename = "find_regexp")]
	FindRegExp(String),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionalDirective {
	As(String),
	Filter(Vec<String>),
	FilterRegExp(Vec<String>),
	IncludeOnly(Vec<String>),
	IncludeOnlyRegExp(Vec<String>),
	FindMatchesFiles(bool),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallDirective {
	source: SourceDirective,
	install_to: String,
	additional: Vec<OptionalDirective>,
} impl InstallDirective {
	pub fn new(source: SourceDirective, install_to: String, additional: Vec<OptionalDirective>) -> Self {
		Self { source, install_to, additional }
	}
}