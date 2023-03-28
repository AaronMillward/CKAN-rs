//! CKAN-rs
//! 
//! A Kerbal Space Program mod manager library.
//! 
//! This library provides a system for browsing, downloading, installing and tracking mods.
//! 
//! # Usage
//! TODO
//! 

pub mod error;
pub use error::Result;
pub use error::Error;

pub mod metadb;
pub use metadb::MetaDB;
pub use metadb::package::Package;

pub mod config;
pub use config::CkanRsConfig;

pub mod installation;
pub mod relationship_resolver;
pub mod game_instance;