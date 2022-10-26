use petgraph::prelude::*;

use crate::metadb::ckan::ModUniqueIdentifier;

pub type DependencyGraph = StableDiGraph<NodeData, EdgeData>;

#[derive(Debug, Clone)]
pub struct CandidateData {
	pub dirty: bool,
	pub id: ModUniqueIdentifier,
}

#[derive(Debug, Clone)]
pub enum NodeData {
	/// Node contains a module which can't be changed.
	Fixed(String, ModUniqueIdentifier),
	/// Node contains a possibly compatible module.
	Candidate(String, CandidateData),
	/// Node only refers to an identifier with no additonal information.
	Stub(String),
	/// Control node for giving the users requests a presence in the graph.
	Meta,

	Decision,
	/// An identifier that does not exist itself, instead uses an anyof edge to represent possibly fulfilling nodes.
	Virtual(String),
}

#[derive(Debug, Clone)]
pub enum EdgeData {
	/// Any of the target nodes can be used to fulfill the source node.
	AnyOf(crate::metadb::ckan::ModVersionBounds),
	/// A requirement from the source module for the target to be within `ModVersionBounds`
	Depends(crate::metadb::ckan::ModVersionBounds),
	/// Source must use the target. mainly exists to handle `Decision` nodes.
	Requires,
	/// Modules inside these bounds are not compatible with the source node.
	Conflicts(crate::metadb::ckan::ModVersionBounds),
	/// Target node has the same requirements as the source.
	SameAs,
	/// Choice outgoing from `Decision` node.
	Selected,
}