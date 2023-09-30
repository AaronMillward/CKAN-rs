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

mod package_tree;
pub use package_tree::PackageTree;
pub use package_tree::Complete;
pub use package_tree::InProgress;
pub use package_tree::ResolverStatus;
pub use package_tree::DeterminePackageError;
pub use package_tree::DecisionInfo;

/// A requirement that can be given to the resolver to fulfill.
#[derive(Debug, Default, Clone)]
pub struct InstallTarget {
	pub identifier: String,
	pub required_version: PackageVersionBounds
}

impl From<crate::metadb::package::PackageIdentifier> for InstallTarget {
	fn from(value: crate::metadb::package::PackageIdentifier) -> Self {
		InstallTarget { identifier: value.identifier, required_version: VersionBounds::Explicit(value.version) }
	}
}