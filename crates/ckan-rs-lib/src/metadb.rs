//! # CKAN's metadb
//! 
//! To improve performance the metadb is converted from it's native format, a series of JSON files to an sqlite database.
//! 
//! 
//! 

mod ckan;

mod generation;

pub use generation::get_latest_archive;

use rusqlite::params;

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

#[cfg(test)]
mod tests {
	#[test]
	fn sql_statements_compile_in_database() {
		let conn = rusqlite::Connection::open_in_memory().expect("failed to open db");
		conn.execute_batch(include_str!("metadb/metadb-schema.sql")).expect("failed to create db tables"); /* Create DB */
		conn.prepare(include_str!("metadb/insert-mod.sql")).unwrap();
		conn.prepare(include_str!("metadb/insert-author.sql")).unwrap();
		conn.prepare(include_str!("metadb/insert-mod-license.sql")).unwrap();
		conn.prepare(include_str!("metadb/insert-mod-author.sql")).unwrap();
		conn.prepare(include_str!("metadb/insert-mod-tag.sql")).unwrap();
		conn.prepare(include_str!("metadb/insert-mod-localization.sql")).unwrap();
		conn.prepare(include_str!("metadb/insert-mod-relationship.sql")).unwrap();
		conn.prepare(include_str!("metadb/insert-identifier.sql")).unwrap();
	}
}