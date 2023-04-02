//! Various helper functions for testing
//! 
//! Functions in this module should use results and not use any panics to avoid confusion in callers.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum TestUtilError {
	#[error("IO error: {0}")]
	IO(#[from] std::io::Error),
	#[error("FSExtra error: {0}")]
	FSExtra(#[from] fs_extra::error::Error),
}

pub fn create_fake_game_instance() -> Result<PathBuf, TestUtilError> {
	let dir = tempfile::tempdir()?;
	let template = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/fake-game-dir");
	fs_extra::dir::copy(template, dir.path(), &fs_extra::dir::CopyOptions::new())?;
	Ok(dir.into_path().join("fake-game-dir"))
}