//! Utilities for getting a valid set of compatible modules to be installed.
//! 
//! This module aims to provide tools which can take a list of requirements from the user and provide a list of modules to install.
//! 
use std::collections::{HashSet, VecDeque};

use crate::metadb::*;
use crate::metadb::ckan::*;
use petgraph::prelude::*;

mod dependency_graph;
use dependency_graph::*;

#[derive(Debug, Default)]
pub struct InstallRequirement {
	pub mod_identifier: String,
	pub required_version: ModVersionBounds
}

#[derive(Debug)]
pub struct DecisionInfo {
	/* TODO: pub help_text: Option<String>, */
	/// Available choices for the decision.
	pub options: Vec<String>,
	/// Which module requires this decision.
	pub source: String,
}

/// These errors halt the progression of the resolver.
#[derive(Debug)]
pub enum DetermineModuleError {
	/// Identifier does not exist at all, not even as a virtual.
	IdentifierDoesNotExist,
	/// There are no modules within the version bounds.
	NoCompatibleVersion,
	/// There are no modules compatible with the game versions.
	NoCompatibleGameVersion,
	/// The versions bounds placed on this module do not have any intersection making them impossible to fulfill.
	VersionBoundsImcompatible,

}

pub enum ResolverStatus {
	/// The resolve has been succsessful, all modules are valid and compatible with each other.
	Complete(Vec<ckan::ModUniqueIdentifier>),
	/// The resolver was not able to determine which module to use when presented with a choice.
	/// Use `RelationshipResolver::add_decision` and `DecisionInfo::options` to provide the resolver with a selection.
	DecisionsRequired(Vec<DecisionInfo>),
	/// The resolver is unable to continue as an unavoidable conflict has occured.
	/// 
	/// It is best to present this information to the user and allow them to decide how to proceed.
	Failed(Vec<(String, DetermineModuleError)>),
}

/// RelationshipResolver will take a list of top level requirements and generate a list of required modules
/// 
/// # Usage
/// First create a resolver using `RelationshipResolver::new`
/// then call `RelationshipResolver::attempt_resolve` until complete while answering any decisions presented.
/// 
/// ## Failures
/// The resolver may fail for reasons described in `DetermineModuleError`
/// when these occur they represent an error the resolver can't solve without human intervention.
#[derive(Debug)]
pub struct RelationshipResolver<'db> {
	metadb: &'db MetaDB,
	/// Tells the resolver which module to chose when faced with a decision
	decisions: HashSet<String>,

	compatible_ksp_versions: Vec<KspVersion>,

	dep_graph: DependencyGraph,
	meta_node: NodeIndex,

	is_complete: bool,
}

impl<'db> RelationshipResolver<'db> {
	/// Creates a new `RelationshipResolver`.
	/// 
	/// # Arguments
	/// - `metadb`: `MetaDB` containing the modules to resolve with.
	/// - `modules_to_install`: A list of requirements to resolve.
	/// - `existing_graph`: if adding to a completed resolve, the completed graph can be passed here to shorten the resolve process.
	/// - `compatible_ksp_versions`: Modules for these versions of the game can be installed.
	pub fn new(metadb: &'db MetaDB, modules_to_install: &[InstallRequirement], existing_graph: Option<DependencyGraph>, compatible_ksp_versions: Vec<KspVersion>) -> Self {
		let mut resolver = RelationshipResolver {
			metadb,
			decisions: Default::default(),
			compatible_ksp_versions,
			dep_graph: existing_graph.unwrap_or_default(),
			meta_node: Default::default(),
			is_complete: false,
		};

		let meta = resolver.dep_graph.add_node(NodeData::Meta);
		resolver.meta_node = meta;

		for m in modules_to_install {
			let new = get_or_add_node_index(&mut resolver.dep_graph, &m.mod_identifier);
			resolver.dep_graph.add_edge(meta, new, EdgeData::Depends(m.required_version.clone()));
		}

		resolver
	}

	/// Run the resolver process until complete or stopped by a decision or failure.
	/// 
	/// See `RelationshipResolver` documentation for more info on the resolve process.
	pub fn attempt_resolve(&mut self) -> ResolverStatus {
		/* Overview of process
		We attempt to uncover as many edges as possible using a breadth first approach. we do this for the following reasons;
		- We are mainly trying to avoid requiring user feedback.
		- Decision nodes can potentially be resolved without user intervention if one of their children is already required.
		- We want to find forseeable unresolvable nodes before asking the user for input.
		 */

		/* TODO: Install Order */


		fn add_node_requirements_to_queue(graph: &mut DependencyGraph, queue: &mut VecDeque::<NodeIndex>, src: NodeIndex) {
			for e in graph.edges_directed(src, Outgoing) {
				match e.weight() {
					EdgeData::Conflicts(_) => continue, /* Conflicts doesn't have to point to a required node */
					EdgeData::AnyOf(_) | EdgeData::Option => continue, /* These don't represent an actual selection */
					EdgeData::Selected | EdgeData::Depends(_) | EdgeData::Decision => {
						queue.push_back(e.target());
					}
				}
			}
		}

		loop {
			/* Breadth First Search */
			let mut failures = Vec::<(NodeIndex, DetermineModuleError)>::new();
			let mut compatible_candidates = Vec::<NodeIndex>::new();
			let mut found_dirty = false;
			let mut pending_decision_nodes = Vec::<NodeIndex>::new();
			let mut visited = Vec::<NodeIndex>::with_capacity(self.dep_graph.node_count());
			let mut queue = VecDeque::<NodeIndex>::new();
			queue.push_front(self.meta_node);
			while let Some(i) = queue.pop_front() {
				if visited.contains(&i) {
					continue;
				}
				visited.push(i);
				let weight = self.dep_graph.node_weight(i).expect("invalid node index in the queue");
				match weight {
					/* Fixed nodes can't be changed so we don't care about their paths. We can assume that every requirement is already fulfilled */
					NodeData::Fixed(_, _) => {},
					NodeData::Candidate(_, data) => {
						if data.dirty {
							found_dirty = true;
							if let Err(e) = self.determine_module_for_candidate(i) { failures.push((i,e)) }
						} else {
							compatible_candidates.push(i);
						}
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, i);
					},
					NodeData::Stub(_) => {
						found_dirty = true;
						if let Err(e) = self.determine_module_for_candidate(i) { failures.push((i,e)) }
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, i); /* A failed stub will have no edges making this work */
					},
					NodeData::Decision => {
						/*
						We wait until after the search has finished to handle these nodes
						because if there are any unresolved nodes they could contain requirements
						to a child allowing us to make an implicit selection.
						*/
						let mut has_selection = false;

						for e in self.dep_graph.edges_directed(i, Outgoing) {
							if let EdgeData::Selected = e.weight() {
								queue.push_back(e.target());
								has_selection = true;
							}
						}
						
						if !has_selection {
							pending_decision_nodes.push(i);
						}
					},
					NodeData::Meta => {
						/* Add all child nodes of the meta requirement, pretty simple. This should only ever happen once at the start of the search. */
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, i);
					},
					NodeData::Virtual(_) => {
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, i);
					},
				}
			}

			/* Post-Search */

			if !failures.is_empty() {
				self.is_complete = false;
				return ResolverStatus::Failed(
					failures.into_iter().map(|(i,e)| {
						let s = get_node_identifier(&self.dep_graph, i).expect("failed to get identifier for node");
						(s.clone(),e)
					}).collect()
				);
			}

			if !found_dirty {
				if pending_decision_nodes.is_empty() {
					/* The resolve is complete */
					self.is_complete = true;
					return ResolverStatus::Complete(
						compatible_candidates.into_iter().map(|candidate| {
							/* XXX: Assume candidate is a `Candidate` node */
							if let NodeData::Candidate(_, data) = &self.dep_graph[candidate] {
								data.id.clone()
							} else {
								/* Should never actually happen we just can't prove it to the compiler */
								unimplemented!("bad node variant");
							}
						}).rev().collect()
						/* TODO: Better install ordering */
						/* XXX: `.rev()` seems like a crude way of getting an install order. There might be some case where a later queue item depends on an earlier one */
					)
				}

				/* Handle Decision Nodes / Determine Selection */
				/* TODO: AlwaysAskDecisions option to disabled implicit decisions */
				/* TODO: Check options for incompatibility before returning decision */
				/* XXX: There might be some way to check the consequences of all options to determine the best resolve */
				let mut explicit_required = Vec::<NodeIndex>::new();
				for decision_node in pending_decision_nodes {
					let mut selections = Vec::<(NodeIndex, NodeIndex)>::new();

					for decision_option in self.dep_graph.edges_directed(decision_node, Outgoing) {
						if let NodeData::Stub(name) | NodeData::Candidate(name, _) = &self.dep_graph[decision_option.target()] {
							if self.decisions.contains(name) {
								selections.push((decision_node,decision_option.target()));
								continue;
							}
						}

						for existing_requirement in self.dep_graph.edges_directed(decision_option.target(), Incoming) {
							match existing_requirement.weight() {
								EdgeData::AnyOf(_) | EdgeData::Option | EdgeData::Conflicts(_) | EdgeData::Decision => continue,
								| EdgeData::Depends(_)
								| EdgeData::Selected => {
									/* XXX: are multiple selections valid? */
									selections.push((decision_node,decision_option.target()));
									break;
								},
							}
						}
					}

					if selections.is_empty() {
						explicit_required.push(decision_node);
					} else {
						for s in selections {
							self.dep_graph.add_edge(s.0, s.1, EdgeData::Selected);
						}
					}
				}

				if !explicit_required.is_empty() {
					self.is_complete = false;
					return ResolverStatus::DecisionsRequired(
						explicit_required.into_iter().map(|i| {
							/* XXX: Assumes i is a Decision node */
	
							let decision_parent_node = &self.dep_graph[self.dep_graph.edges_directed(i, petgraph::Incoming).next().expect("floating decision node").source()];
							let source = if let NodeData::Stub(name) | NodeData::Candidate(name, _) | NodeData::Virtual(name) = decision_parent_node {
								name.clone()
							} else {
								unimplemented!("decision node attached to incorrect node type")
							};
	
							
							let mut  options = Vec::<String>::new();
							for e in self.dep_graph.edges_directed(i, Outgoing) {
								let target = &self.dep_graph[e.target()];
	
								if let NodeData::Stub(name) | NodeData::Candidate(name, _) | NodeData::Virtual(name) = target {
									options.push(name.clone());
								} else {
									unimplemented!("incorrect decision child node type");
								}
							}
							 
							DecisionInfo { options, source }
						}).collect::<Vec<_>>()
					);
				}
			}
		}
	}

	/// Reads the requirements placed on `src` and gets the latest module meeting the requirements.
	/// 
	/// This function makes only finds the latest candidate compatible with the node requirements.
	/// It makes no attempt to resolve conflicts arising from this choice.
	/// # Panics
	/// - If `src` is not a `Candidate` or `Stub`.
	fn determine_module_for_candidate(&mut self, src: NodeIndex) -> Result<(), DetermineModuleError> {
		if let NodeData::Candidate(name, _) | NodeData::Stub(name) = &self.dep_graph[src] {
			let name = name.clone();

			let bounds = get_version_bounds_for_node(&self.dep_graph, src).ok_or(DetermineModuleError::VersionBoundsImcompatible)?;

			/* We don't use `bounds` yet, we want to grab every module providing the identifier so we can tell if it exists at all */
			let matching_modules_providing = self.metadb.get_modules().iter().get_modules_providing(&ModuleDescriptor { name: name.clone(), version: VersionBounds::Any });
			if matching_modules_providing.is_empty() {
				Err(DetermineModuleError::IdentifierDoesNotExist)
			} else if matching_modules_providing.len() == 1 {
				/* There's only one module providing so it's a real module */

				let mut m: Vec<_> = matching_modules_providing.into_values().next().unwrap().into_iter().mod_version_matches(bounds).collect();
				if m.is_empty() { return Err(DetermineModuleError::NoCompatibleVersion); }
				m = m.into_iter().ksp_version_matches(self.compatible_ksp_versions.clone()).collect();
				if m.is_empty() { return Err(DetermineModuleError::NoCompatibleGameVersion); }

				/* 
				This is the latest module that matches the requirements.
				It's not for this function to determine the later side effects of this decision,
				So we don't need to track the attempted modules or iterate all possible candidates.
				*/

				m.sort();
				let latest = m.get(0).cloned().unwrap();

				set_node_as_module(&mut self.dep_graph, src, latest);
				
				Ok(())
			} else {
				/* The module is virtual */
				/* We represent the providers as a `Decision` node */
				self.dep_graph[src] = NodeData::Virtual(name);
				let decision = self.dep_graph.add_node(NodeData::Decision);
				self.dep_graph.add_edge(src, decision, EdgeData::Decision);
				for k in matching_modules_providing.keys() {
					let provider = get_or_add_node_index(&mut self.dep_graph, k);
					self.dep_graph.add_edge(decision, provider, EdgeData::Option);
				}
				Ok(())
			}
		} else {
			unimplemented!("node type can't become a candidate type")
		}
	}

	/// Adds an identifer to be selected when present in a decisions options.
	/// 
	/// This can be done at any point in the resolve process and may be required to continue the resolve.
	pub fn add_decision(&mut self, identifier: &str) {
		self.decisions.insert(identifier.to_owned());
	}

	/// Gets the graph of the resolver if it has been completed, otherwise returns the resolver in `Err`
	pub fn get_complete_graph(self) -> Result<DependencyGraph, Self> {
		if self.is_complete {
			Ok(self.dep_graph)
		} else {
			Err(self)
		}
	}

	/// Get the dependency graph regardless of state.
	/// 
	/// This is mainly for debugging purposes, it is not recommended to analyse this graph to complete the resolve.
	/// Especially as the layout of this graph could change in the future.
	pub fn get_graph(&self) -> &DependencyGraph {
		/* XXX: If callers are using this function to complete resolves we should consider this a fault of the resolver for not providing the required functionality */
		&self.dep_graph
	}
}