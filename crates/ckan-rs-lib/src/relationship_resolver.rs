//! Utilities for getting a valid set of compatible packages to be installed from a list of desired packages.

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
mod finalized_resolver;
pub use finalized_resolver::ResolverFinalized;

#[derive(Debug, Default, Clone)]
pub struct InstallRequirement {
	pub identifier: String,
	pub required_version: PackageVersionBounds
}