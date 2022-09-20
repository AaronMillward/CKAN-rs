pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	ReqwestError(reqwest::Error),
	IOError(std::io::Error),
	SerdeJSONError(serde_json::Error),
	SQLError(rusqlite::Error),
	ParseError(String),
	ValidationError(String),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error {}
impl From<reqwest::Error>    for Error { fn from(e: reqwest::Error)    -> Self { Error::ReqwestError(e) } }
impl From<std::io::Error>    for Error { fn from(e: std::io::Error)    -> Self { Error::IOError(e) } }
impl From<serde_json::Error> for Error { fn from(e: serde_json::Error) -> Self { Error::SerdeJSONError(e) } }
impl From<rusqlite::Error>   for Error { fn from(e: rusqlite::Error)   -> Self { Error::SQLError(e) } }