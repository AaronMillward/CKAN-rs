//!
//! 

use petgraph::prelude::*;

use super::DependencyGraph;
use super::NodeData;

pub struct ResolverFinalized {
	dep_graph: DependencyGraph,
	candidates: Vec<NodeIndex>,
}

impl ResolverFinalized {
	pub(super) fn new(dep_graph: DependencyGraph, candidates: Vec<NodeIndex>) -> Self {
		Self {
			dep_graph,
			candidates,
		}
	}

	pub fn get_graph(self) -> DependencyGraph {
		self.dep_graph
	}

	pub fn get_new_packages(&self) -> Vec<crate::metadb::ckan::PackageIdentifier> {
		self.candidates.iter().map(|candidate| {
			/* XXX: Assume candidate is a `Candidate` node */
			if let NodeData::Candidate(_, data) = &self.dep_graph[*candidate] {
				data.id.clone()
			} else {
				/* Should never actually happen we just can't prove it to the compiler */
				unimplemented!("bad node variant");
			}
		}).rev().collect()
		/* TODO: Better install ordering */
		/* XXX: `.rev()` seems like a crude way of getting an install order. There might be some case where a later queue item depends on an earlier one */
	}
}
