//! # CKAN's metadb
//! 
//! To improve performance the metadb is converted from it's native format, a series of JSON files to an sqlite database.
//! 
//! 
//! 

mod ckan;

mod generation;

pub use generation::get_latest_archive;

pub struct MetaDB {
	connection: rusqlite::Connection,
}

impl MetaDB {
	pub fn open(path: &std::path::Path) -> crate::Result<Self> {
		Ok(MetaDB {
			connection: rusqlite::Connection::open(path)?,
		})
	}
}