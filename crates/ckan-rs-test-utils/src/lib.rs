//! Various helper functions for testing
//! 
//! functions in this module should use results and not use any panics to avoid confusion in callers

use std::io::{Read, Write};

/// Gets a MetaDB for use in testing
/// # Parameters
/// - `serialized_path` - When present db is read from the path, if not the latest archive is grabbed and converted
pub fn get_metadb(serialized_path: Option<std::path::PathBuf>) -> ckan_rs::MetaDB {
	use ckan_rs::MetaDB;

	if let Some(path) = serialized_path {
		let mut f = std::fs::File::open(path).expect("failed to open metadb file");
		let mut v = Vec::<u8>::new();
		f.read_to_end(&mut v).unwrap();
		bincode::deserialize::<MetaDB>(&v).expect("failed to deserialize db")
	} else {
		let archive_data = ckan_rs::metadb::get_latest_archive().expect("failed to download archive");
		let mut gz = flate2::bufread::GzDecoder::new(archive_data.as_slice());
		let mut v = Vec::<u8>::new();
		gz.read_to_end(&mut v).unwrap();
		let db: MetaDB = MetaDB::generate_from_archive(
			&mut tar::Archive::new(v.as_slice()), 
			true
		).expect("failed to create db");

		let data = bincode::serialize(&db).unwrap();
		let mut f = std::fs::File::create("metadb-new.bin").unwrap();
		f.write_all(&data).unwrap();

		db
	}
}