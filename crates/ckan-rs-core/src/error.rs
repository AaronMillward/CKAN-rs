//! Library error type.

pub type Result<T> = std::result::Result<T, Error>;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("reqwest error: {0}")]
	Reqwest(#[from] reqwest::Error),
	#[error("IO error: {0}")]
	IO(#[from] std::io::Error),
	#[error("JSON error: {0}")]
	SerdeJSON(#[from] serde_json::Error),
	#[error("bincode error: {0}")]
	Bincode(#[from] bincode::Error),
	#[error("parsing error: {0}")]
	Parse(String),
	#[error("validation error: {0}")]
	Validation(String),
	#[error("selection invalid")]
	InvalidSelection,
	#[error("downloader failed")]
	Download(#[from] crate::installation::download::DownloadError),
	#[error("already exists")]
	AlreadyExists,
}