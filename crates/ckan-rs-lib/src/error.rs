pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	Reqwest(reqwest::Error),
	IO(std::io::Error),
	SerdeJSON(serde_json::Error),
	Parse(String),
	Validation(String),
	InvalidSelection,
	Acquirement(crate::installer::retrieval::RetrievalError),
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

error_wrapper!(Error, Error::Reqwest  , reqwest::Error);
error_wrapper!(Error, Error::IO       , std::io::Error);
error_wrapper!(Error, Error::SerdeJSON, serde_json::Error);
error_wrapper!(Error, Error::Acquirement, crate::installer::retrieval::RetrievalError);