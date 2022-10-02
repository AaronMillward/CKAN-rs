pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	ReqwestError(reqwest::Error),
	IOError(std::io::Error),
	SerdeJSONError(serde_json::Error),
	ParseError(String),
	ValidationError(String),
	InvalidSelection,
	IncompatibleGameVersion,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error {}

macro_rules! error_wrapper(
	($t:ty, $e:expr) => (
		impl From<$t> for Error { fn from(e: $t) -> Self { $e(e) } }
	)
);

error_wrapper!(reqwest::Error    , Error::ReqwestError);
error_wrapper!(std::io::Error    , Error::IOError);
error_wrapper!(serde_json::Error , Error::SerdeJSONError);