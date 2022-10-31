pub mod error;
pub use error::Result;
pub use error::Error;

pub mod metadb;
pub use metadb::MetaDB;
pub use metadb::ckan::ModuleInfo;

pub mod config;
pub use config::CkanRsOptions;

pub mod manager;

pub mod installer;
