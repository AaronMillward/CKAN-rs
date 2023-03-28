use serde::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceDirective {
	/// The file or directory root that this directive pertains to.
	File(String),
	/// Locate the top-most directory which exactly matches the name specified.
	Find(String),
	/// Locate the top-most directory which matches the specified regular expression.
	FindRegExp(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionalDirective {
	/// The name to give the matching directory or file when installed.
	As(String),
	/// A string, or list of strings, of file parts that should *not* be installed. These are treated as literal things which must match a file name or directory. Examples of filters may be Thumbs.db, or Source. Filters are considered case-insensitive.
	Filter(Vec<String>),
	/// A string, or list of strings, which are treated as case-sensitive C# regular expressions which are matched against the full paths from the installing zip-file. If a file matches the regular expression, it is not installed.
	FilterRegExp(Vec<String>),
	/// A string, or list of strings, of file parts that should be installed. These are treated as literal things which must match a file name or directory. Examples of this may be Settings.cfg, or Plugin. These are considered case-insensitive.
	IncludeOnly(Vec<String>),
	/// A string, or list of strings, which are treated as case-sensitive C# regular expressions which are matched against the full paths from the installing zip-file. If a file matches the regular expression, it is installed.
	IncludeOnlyRegExp(Vec<String>),
	/// If set to true then both find and find_regexp will match files in addition to directories.
	FindMatchesFiles(bool),
}

/// Describes how to install the content of a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallDirective {
	pub source: SourceDirective,
	pub install_to: String,
	pub additional: Vec<OptionalDirective>,
} 

impl InstallDirective {
	pub fn new(source: SourceDirective, install_to: String, additional: Vec<OptionalDirective>) -> Self {
		Self { source, install_to, additional }
	}
}