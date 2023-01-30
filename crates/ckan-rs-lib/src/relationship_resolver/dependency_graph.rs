//! Module for only DependencyGraph functions not related to the overall resolving process.

use petgraph::prelude::*;

use crate::metadb::*;
use crate::metadb::ckan::*;

pub type DependencyGraph = StableDiGraph<NodeData, EdgeData>;

#[derive(Debug, Clone)]
pub struct CandidateData {
	pub dirty: bool,
	pub id: PackageIdentifier,
}

#[derive(Debug, Clone)]
pub enum NodeData {
	/// Node contains a package which can't be changed.
	Fixed(String, PackageIdentifier),
	/// Node contains a possibly compatible package.
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
	AnyOf(crate::metadb::ckan::PackageVersionBounds),
	/// A requirement from the source package for the target to be within `ModVersionBounds`
	Depends(crate::metadb::ckan::PackageVersionBounds),
	/// Leads to a `Decision` node
	Decision,
	/// A possible choice for a `Decision` node.
	/// 
	/// Not be confused with `AnyOf` which is also used in `Decision` nodes.
	Option,
	/// Packages inside these bounds are not compatible with the source node.
	Conflicts(crate::metadb::ckan::PackageVersionBounds),
	/// Choice outgoing from `Decision` node.
	Selected,
}

/* TODO: functions for adding and removing nodes and edges according to the rules. */

/// Analyses version incoming requirements to find a bound that satisfies.
/// 
/// This function will recurse to get version bounds for virtual nodes.
pub fn get_version_bounds_for_node(graph: &DependencyGraph, src: NodeIndex) -> Option<PackageVersionBounds> {
	/* TODO: Conflicts */
	let mut bound = VersionBounds::Any;

	for e in graph.edges_directed(src, petgraph::Incoming) {
		if let EdgeData::Depends(vb) = e.weight() {
			/* Impossible to fulfill requirements */
			bound = bound.inner_join(vb)?;
		} else if let EdgeData::Selected = e.weight() {
			/* We have to do this loop because find_edge only returns 1 result */
			for e2 in graph.edges_directed(src, petgraph::Incoming) {
				if e2.source() == e.source() {
					/* e and e2 source are `Decision` nodes */
					if let EdgeData::AnyOf(vb) = e2.weight() {
						/* Impossible to fulfill requirements */
						bound = bound.inner_join(vb)?;
					} else if let EdgeData::Option = e2.weight() {
						/* Run up the graph to get the requirements */
						/* XXX: This is assumed to be a `Decision` node and should only have 1 incoming node  */
						let x = get_version_bounds_for_node(
							graph,
							graph.edges_directed(e2.source(), petgraph::Incoming).next().expect("floating decision node").source()
						)?;
						bound = bound.inner_join(&x)?;
					}
				}
			}
		}
	}

	Some(bound)
}

/// Clears `src` outbound edges and replaces them with edges from `package`
/// # Panics
/// - If `src` is not a `Candidate` or `Stub`.
pub fn set_node_as_package(graph: &mut DependencyGraph, src: NodeIndex, package: &ckan::Package) {
	/* TODO: Check if any requirements actually changed. without this check cyclic dependencies will repeatedly set each other as dirty */
	let id = if let NodeData::Candidate(name, _) | NodeData::Stub(name) = &graph[src] {
		name.clone()
	} else {
		unimplemented!("node can't be set as a package.")
	};

	clear_nodes_requirements(graph, src);
	add_node_edges_from_package(graph, package, src);
	graph[src] = NodeData::Candidate(id, CandidateData { dirty: false, id: package.identifier.clone() } );
}


/// Add all the required edges and `Decision` nodes from `package` to `src`
/// - Sets the `dirty` flag on any candidate nodes affected.
/// - Does not remove existing edges of the node. (See `clear_nodes_requirements()`)
/// - Is not concerned with the type of node it is being applied to.
pub fn add_node_edges_from_package(graph: &mut DependencyGraph, package: &ckan::Package, src: NodeIndex) {
	for req in &package.depends {
		match req {
			Relationship::AnyOf(r) => {
				let decision = graph.add_node(NodeData::Decision);
				graph.add_edge(src, decision, EdgeData::Decision);
				for b_desc in r {
					let b = get_or_add_node_index(graph, &b_desc.name);
					if let NodeData::Candidate(_, data) = &mut graph[b] { data.dirty = true; }
					graph.add_edge(decision, b, EdgeData::AnyOf(b_desc.version.clone()));
				}
			},
			Relationship::One(b_desc) => {
				let b = get_or_add_node_index(graph, &b_desc.name);
				if let NodeData::Candidate(_, data) = &mut graph[b] { data.dirty = true; }
				graph.add_edge(src, b, EdgeData::Depends(b_desc.version.clone()));
			},
		}
	}

	for conflict in &package.conflicts {
		for r in conflict.as_vec() {
			let b = get_or_add_node_index(graph, &r.name);
			if let NodeData::Candidate(_, data) = &mut graph[b] { data.dirty = true; }
			graph.add_edge(src, b, EdgeData::Conflicts(r.version.clone()));
		}
	}
}

/// Removes all out going connections from `src` including `Decision` nodes attached to it.
/// - Sets the `dirty` flag on any candidate nodes affected.
/// - This method is also not concerned with the type of node it is being applied to.
pub fn clear_nodes_requirements(graph: &mut DependencyGraph, src: NodeIndex) {
	for id in graph.edges_directed(src, Outgoing).map(|e| e.id()).collect::<Vec<_>>() {
		let (_, target) = graph.edge_endpoints(id).unwrap();
		if let NodeData::Candidate(_, data) = &mut graph[target] { data.dirty = true; }
		if matches!(graph[target], NodeData::Decision) {
			/* XXX: Does this remove all the nodes edges? doc is unclear */
			graph.remove_node(target);
		}
		graph.remove_edge(id);
	}
}

pub fn get_node_index(graph: &mut DependencyGraph, node: &String) -> Option<NodeIndex> {
	graph.node_weights()
		.enumerate()
		.find(|(_, data)| {
			match data {
				NodeData::Fixed(id, _) 
				| NodeData::Candidate(id, _)
				| NodeData::Stub(id)
				| NodeData::Virtual(id) => id == node,

				NodeData::Meta => false,
				NodeData::Decision => false,
			}
		})
		.map(|(i,_)| petgraph::graph::node_index(i))
}

/// Returns the index of the existing node or a `Stub` node with `name`
pub fn get_or_add_node_index(graph: &mut DependencyGraph, name: &String) -> NodeIndex {
	get_node_index(graph, name)
		.unwrap_or_else(|| graph.add_node(NodeData::Stub(name.clone())))
}

pub fn get_node_identifier(graph: &DependencyGraph, src: NodeIndex) -> Option<&String> {
	let weight = graph.node_weight(src)?;
	if let NodeData::Stub(name) | NodeData::Candidate(name, _) | NodeData::Virtual(name) | NodeData::Fixed(name, _) = weight {
		Some(name)
	} else {
		None
	}
}