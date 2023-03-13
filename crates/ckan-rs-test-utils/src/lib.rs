//! Various helper functions for testing
//! 
//! Functions in this module should use results and not use any panics to avoid confusion in callers.

use std::path::PathBuf;

#[derive(Debug)]
pub enum TestUtilError {
	IO(std::io::Error),
	FSExtra(fs_extra::error::Error),
}

impl std::fmt::Display for TestUtilError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for TestUtilError {}

#[macro_export]
macro_rules! error_wrapper(
	($errortype:ty, $error:expr, $t:ty) => (
		impl From<$t> for $errortype { fn from(e: $t) -> Self { $error(e) } }
	)
);

error_wrapper!(TestUtilError, TestUtilError::IO, std::io::Error);
error_wrapper!(TestUtilError, TestUtilError::FSExtra, fs_extra::error::Error);

pub fn create_fake_game_instance() -> Result<PathBuf, TestUtilError> {
	let dir = tempfile::tempdir()?;
	let template = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/fake-game-dir");
	fs_extra::dir::copy(template, dir.path(), &fs_extra::dir::CopyOptions::new())?;
	Ok(dir.into_path().join("fake-game-dir"))
}