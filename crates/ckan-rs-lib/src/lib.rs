//! CKAN-rs
//! 
//! A Kerbal Space Program mod manager library.
//! 
//! This library provides a system for browsing, downloading, installing and tracking mods.
//! 
//! # Usage
//! 1. Load or generate the [`MetaDB`] this is the package repository used throughout the program.
//! 1. Create or load a [`GameInstance`](game_instance::GameInstance).
//! 1. See [`relationship_resolver`] for how to determine required packages for a given target.
//! 1. [Extract](installation::content::extract_content_to_deployment) the packages contents.
//! 1. [Enable](game_instance::GameInstance::enable_package) the packages.
//! 1. [Redeploy](game_instance::GameInstance::redeploy_packages) the packages.

pub mod error;
pub use error::Result;
pub use error::Error;

pub mod metadb;
pub use metadb::MetaDB;

pub mod config;
pub use config::CkanRsConfig;

pub mod installation;
pub mod relationship_resolver;
pub mod game_instance;