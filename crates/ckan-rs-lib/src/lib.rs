pub mod error;
pub mod metadb;
pub use metadb::MetaDB;
pub use metadb::ckan::ModuleInfo;

pub mod manager;

pub use error::Result;
pub use error::Error;