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

pub struct DecisionInfo {
	/* TODO: pub help_text: Option<String>, */
	/// Available choices for the decision.
	pub options: Vec<String>,
	/// Which module requires this decision.
	pub source: String,
}

pub enum ResolverStatus {
	Complete,
	DecisionsRequired(Vec<DecisionInfo>),
	Failed,
}

/// RelationshipResolver will take a list of top level requirements and generate a list of required modules
#[derive(Debug)]
pub struct RelationshipResolver<'db> {
	metadb: &'db MetaDB,
	/// Tells the resolver which module to chose when faced with a decision
	decisions: HashSet<String>,

	compatible_ksp_versions: Vec<KspVersion>,

	dep_graph: DependencyGraph,
	meta_node: NodeIndex,
}

impl<'db> RelationshipResolver<'db> {
	pub fn new(metadb: &'db MetaDB, modules_to_install: &'db [InstallRequirement], existing_graph: Option<DependencyGraph>, compatible_ksp_versions: Vec<KspVersion>) -> Self {
		let mut resolver = RelationshipResolver {
			metadb,
			decisions: Default::default(),
			compatible_ksp_versions,
			dep_graph: existing_graph.unwrap_or_default(),
			meta_node: Default::default(),
		};

		let meta = resolver.dep_graph.add_node(NodeData::Meta);
		resolver.meta_node = meta;

		for m in modules_to_install {
			let new = get_or_add_node_index(&mut resolver.dep_graph, &m.mod_identifier);
			resolver.dep_graph.add_edge(meta, new, EdgeData::Depends(m.required_version.clone()));
		}

		resolver
	}

	pub fn attempt_resolve(&mut self) -> ResolverStatus {
		/* Overview of process
		We attempt to uncover as many edges as possible using a breadth first approach. we do this for the following reasons;
		- We are mainly trying to avoid requiring user feedback.
		- Decision nodes can potentially be resolved without user intervention if one of their children is already required.
		- We want to find forseeable unresolvable nodes before asking the user for input.
		 */

		fn add_node_requirements_to_queue(graph: &mut DependencyGraph, queue: &mut VecDeque::<NodeIndex>, visited: &[NodeIndex], src: NodeIndex) {
			for e in graph.edges_directed(src, Outgoing) {
				match e.weight() {
					EdgeData::Conflicts(_) => continue, /* Conflicts doesn't have to point to a required node */
					EdgeData::AnyOf(_) | EdgeData::Option => continue, /* These don't represent an actual selection */
					EdgeData::Selected | EdgeData::Depends(_) | EdgeData::Decision => {
						if !visited.contains(&e.target()) {
							queue.push_back(e.target());
						}
					}
				}
			}
		}

		loop {
			/* Breadth First Search */
			/* TODO: failed should be a vec of node failures */
			let mut failed = false;
			let mut found_dirty = false;
			let mut pending_decision_nodes = Vec::<NodeIndex>::new();
			let mut visited = Vec::<NodeIndex>::with_capacity(self.dep_graph.node_count());
			let mut queue = VecDeque::<NodeIndex>::new();
			queue.push_front(self.meta_node);
			while let Some(i) = queue.pop_front() {
				visited.push(i);
				let weight = self.dep_graph.node_weight(i).expect("An invalid node index was added to the queue");
				match weight {
					/* Fixed nodes can't be changed so we don't care about their paths. We can assume that every requirement is already fulfilled */
					NodeData::Fixed(_, _) => {},
					NodeData::Candidate(_, data) => {
						if data.dirty {
							found_dirty = true;
							failed |= self.determine_module_for_candidate(i).is_err();
						}
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, &visited, i);
					},
					NodeData::Stub(_) => {
						failed |= self.determine_module_for_candidate(i).is_err();
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, &visited, i); /* A failed stub will have no edges making this work */
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
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, &visited, i);
					},
					NodeData::Virtual(_) => {
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, &visited, i);
					},
				}
			}

			/* Post-Search */

			if failed {
				/* TODO: Analyse the error */
				return ResolverStatus::Failed;
			}

			if !found_dirty {
				if pending_decision_nodes.is_empty() {
					/* The resolve is complete */
					return ResolverStatus::Complete; /* TODO: attach list of modules */
				}

				/* Handle Decision Nodes / Determine Selection */
				/* TODO: AlwaysAskDecisions option to disabled implicit decisions */
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
					let infos = explicit_required.into_iter().map(|i| {
						/* Assume i is a Decision node */

						let decision_parent_node = &self.dep_graph[self.dep_graph.edges_directed(i, petgraph::Incoming).next().expect("floating decision node").source()];
						let source = if let NodeData::Stub(name) | NodeData::Candidate(name, _) | NodeData::Virtual(name) = decision_parent_node {
							name.clone()
						} else {
							panic!("decision node attached to incorrect node type")
						};

						
						let mut  options = Vec::<String>::new();
						for e in self.dep_graph.edges_directed(i, Incoming) {
							let target = &self.dep_graph[e.target()];

							if let NodeData::Stub(name) | NodeData::Candidate(name, _) | NodeData::Virtual(name) = target {
								options.push(name.clone());
							} else {
								panic!("incorrect decision child node type");
							}
						}
						 
						DecisionInfo { options, source }
					}).collect::<Vec<_>>();

					return ResolverStatus::DecisionsRequired(infos);
				}
			}
		}
	}

	/// Reads the requirements placed on `src` and gets the latest module meeting the requirements.
	/// # Panics
	/// - If `src` is not a `Candidate` or `Stub`.
	/* TODO: Actual error type for richer fail info */
	fn determine_module_for_candidate(&mut self, src: NodeIndex) -> Result<(), ()> {
		if let NodeData::Candidate(name, _) | NodeData::Stub(name) = &self.dep_graph[src] {
			let name = name.clone();

			let bounds = get_version_bounds_for_node(&self.dep_graph, src).ok_or(())?;

			/* We don't use `bounds` yet, we want to grab every module providing the identifier so we can tell if it exists at all */
			let matching_modules_providing = self.metadb.get_modules().iter().get_modules_providing(&ModuleDescriptor { name: name.clone(), version: VersionBounds::Any });
			if matching_modules_providing.is_empty() {
				/* Identifier does not exist at all, not even as a virtual */
				Err(())
			} else if matching_modules_providing.len() == 1 {
				/* There's only one module providing so it's a real module */

				let mut m: Vec<_> = matching_modules_providing.into_values().next().unwrap().into_iter().mod_version_matches(bounds).collect();
				if m.is_empty() { /* There are no modules within the version bounds */ return Err(()); }
				m = m.into_iter().ksp_version_matches(self.compatible_ksp_versions.clone()).collect();
				if m.is_empty() { /* There are no modules compatible with the game versions */ return Err(()); }

				/* TODO: Track already attempted modules and move down list. This means currently we don't actually try other module versions */
				m.sort();
				let latest = m.get(0).cloned().unwrap();

				set_node_as_module(&mut self.dep_graph, src, latest);
				
				Ok(())
			} else {
				/* The module is virtual */
				/* We represent the providers as a `Decision` node */
				*self.dep_graph.node_weight_mut(src).unwrap() = NodeData::Virtual(name);
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

	pub fn add_decision(&mut self, identifier: &str) {
		self.decisions.insert(identifier.to_owned());
	}
}