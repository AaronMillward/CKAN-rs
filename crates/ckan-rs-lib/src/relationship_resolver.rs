//! Utilities for getting a valid set of compatible packages to be installed from a list of desired packages.
//! 
//! # Usage
//! 1. Create a [`ResolverBuilder`]
//! 1. Use the builder to add package requirements or game versions.
//! 1. [`ResolverBuilder::build()`] to get a [`ResolverProcessor`]
//! 1. [`ResolverProcessor::attempt_resolve()`] until complete while answering any decisions presented
//! by [`ResolverStatus::DecisionsRequired`] by calling [`ResolverProcessor::add_decision()`] with a choice from the decisions vector.
//! 1. [`ResolverProcessor::finalize()`] to get a [`ResolverFinalized`] to query.
//! 1. [`ResolverFinalized::get_new_packages()`] to list all new packages to be installed.

use std::collections::{HashSet, VecDeque};

use crate::metadb::*;
use crate::metadb::package::*;
use petgraph::prelude::*;

mod dependency_graph;
use dependency_graph::*;

mod resolver_builder;
pub use resolver_builder::ResolverBuilder;
mod processing_resolver;
pub use processing_resolver::ResolverProcessor;
pub use processing_resolver::ResolverStatus;
pub use processing_resolver::DeterminePackageError;
mod finalized_resolver;
pub use finalized_resolver::ResolverFinalized;

/// A requirement that can be given to the resolver to fulfill.
#[derive(Debug, Default, Clone)]
pub struct InstallRequirement {
	pub identifier: String,
	pub required_version: PackageVersionBounds
}