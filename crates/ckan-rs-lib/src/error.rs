pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	ReqwestError(reqwest::Error),
	IOError(std::io::Error),
	SerdeJSONError(serde_json::Error),
	ParseError(String),
	ValidationError(String),
	InvalidSelection,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error {}

#[macro_export]
macro_rules! error_wrapper(
	($errortype:ty, $error:expr, $t:ty) => (
		impl From<$t> for $errortype { fn from(e: $t) -> Self { $error(e) } }
	)
);

error_wrapper!(Error, Error::ReqwestError  , reqwest::Error);
error_wrapper!(Error, Error::IOError       , std::io::Error);
error_wrapper!(Error, Error::SerdeJSONError, serde_json::Error);