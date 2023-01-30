//! Kerbal Space Program mod manager library
//! 
//! Provides a system for browsing, downloading, installing and tracking mods.
//! 
//! # Usage
//! 
//! 1. Start by creating or loading a [`game_instance::GameInstance`].
//! 1. Also create or load A [`metadb::MetaDB`] instance.
//! 1. Then call `start_transaction` on the [`game_instance::GameInstance`]
//! 1. Call `commit` on the [`game_instance::GameInstanceTransaction`] and observe the returned value adding decisions as required.
//! 1. Install the list of mods using the utilities in [`installer`] or [`easy_installer`]
//! 

pub mod error;
pub use error::Result;
pub use error::Error;

pub mod metadb;
pub use metadb::MetaDB;
pub use metadb::ckan::Package;

pub mod config;
pub use config::CkanRsOptions;

pub mod installer;
pub mod relationship_resolver;
pub mod game_instance;