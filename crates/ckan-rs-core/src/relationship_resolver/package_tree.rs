use petgraph::visit::IntoNodeReferences;
use serde::{Serialize, Deserialize};

use super::*;

#[derive(Debug)]
pub struct DecisionInfo {
	/* TODO: pub help_text: Option<String>, */
	/// Available choices for the decision.
	pub options: Vec<String>,
	/// Which package requires this decision.
	pub source: String,
}

/// These errors halt the progression of the resolver.
#[derive(Debug, thiserror::Error)]
pub enum DeterminePackageError {
	/// Identifier does not exist at all, not even as a virtual.
	#[error("identifier not found.")]
	IdentifierDoesNotExist,
	/// There are no packages within the version bounds.
	#[error("no package found with the required versions")]
	NoCompatibleVersion,
	/// There are no packages compatible with the game versions.
	#[error("no package is compatible with the required game version.")]
	NoCompatibleGameVersion,
	/// The versions bounds placed on this package do not have any intersection making them impossible to fulfill.
	#[error("version requirements impossible to fulfill.")]
	VersionBoundsImcompatible,
}

/// The state of the resolve in progress.
pub enum ResolverStatus {
	/// The resolve has been succsessful, all packages are valid and compatible with each other.
	Complete,
	/// The resolver was not able to determine which package to use when presented with a choice.
	/// Use `RelationshipResolver::add_decision` and `DecisionInfo::options` to provide the resolver with a selection.
	DecisionsRequired(Vec<DecisionInfo>),
	/// The resolver is unable to continue as an unavoidable conflict has occured.
	/// 
	/// It is best to present this information to the user and allow them to decide how to proceed.
	Failed(Vec<(String, DeterminePackageError)>),
}

#[derive(Debug, Clone)]
pub struct Complete;
#[derive(Debug, Clone)]
pub struct InProgress;

/// Performs the resolve task.
/// 
/// # Errors
/// The resolver may fail for reasons described in [`DeterminePackageError`]
/// when these occur they represent an error the resolver can't solve without human intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTree<State> {
	/// Tells the resolver which package to chose when faced with a decision
	decisions: HashSet<String>,

	compatible_ksp_versions: Vec<KspVersionReal>,

	dep_graph: DependencyGraph,
	compatible_candidates: Vec<NodeIndex>,

	is_complete: bool,

	state: std::marker::PhantomData<State>,
}

impl PackageTree<InProgress> {
	/// Run the resolver process until complete or stopped by a decision or failure.
	/// 
	/// See [`relationship_resolver`](crate::relationship_resolver) documentation for more info on the resolve process.
	pub fn attempt_resolve(&mut self, metadb: &crate::metadb::MetaDB) -> ResolverStatus {
		/* Overview of process
		We attempt to uncover as many edges as possible using a breadth first approach. we do this for the following reasons;
		- We are mainly trying to avoid requiring user feedback.
		- Decision nodes can potentially be resolved without user intervention if one of their children is already required.
		- We want to find forseeable unresolvable nodes before asking the user for input.
		 */

		/* TODO: Install Order */


		fn add_node_requirements_to_queue(graph: &mut DependencyGraph, queue: &mut VecDeque::<NodeIndex>, src: NodeIndex) {
			for e in graph.graph.edges_directed(src, Outgoing) {
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
			let mut failures = Vec::<(NodeIndex, DeterminePackageError)>::new();
			self.compatible_candidates = Vec::<NodeIndex>::new();
			let mut pending_decision_nodes = Vec::<NodeIndex>::new();
			let mut found_dirty = false;
			
			let mut visited = Vec::<NodeIndex>::with_capacity(self.dep_graph.graph.node_count());
			let mut queue = VecDeque::<NodeIndex>::new();
			
			queue.push_front(self.dep_graph.meta_node);
			while let Some(i) = queue.pop_front() {
				if visited.contains(&i) { continue; }
				visited.push(i);
				
				let weight = self.dep_graph.graph.node_weight(i).expect("invalid node index in the queue");
				match weight {
					/* Fixed nodes can't be changed so we don't care about their paths. We can assume that every requirement is already fulfilled */
					NodeData::Fixed(_, _) => {},
					NodeData::Candidate(_, data) => {
						if data.dirty {
							found_dirty = true;
							if let Err(e) = self.determine_package_for_candidate(metadb, i) { failures.push((i,e)) }
						} else {
							self.compatible_candidates.push(i);
						}
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, i);
					},
					NodeData::Stub(_) => {
						found_dirty = true;
						if let Err(e) = self.determine_package_for_candidate(metadb, i) { failures.push((i,e)) }
						add_node_requirements_to_queue(&mut self.dep_graph, &mut queue, i); /* A failed stub will have no edges making this work */
					},
					NodeData::Decision => {
						/*
						We wait until after the search has finished to handle these nodes
						because if there are any unresolved nodes they could contain requirements
						to a child allowing us to make an implicit selection.
						*/
						let mut has_selection = false;

						for e in self.dep_graph.graph.edges_directed(i, Outgoing) {
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
						let s = self.dep_graph.get_node_identifier(i).expect("failed to get identifier for node");
						(s.clone(),e)
					}).collect()
				);
			}

			if !found_dirty {
				if pending_decision_nodes.is_empty() {
					/* The resolve is complete */
					self.is_complete = true;
					return ResolverStatus::Complete
				}

				/* Handle Decision Nodes / Determine Selection */
				/* TODO: AlwaysAskDecisions option to disabled implicit decisions */
				/* TODO: Check options for incompatibility before returning decision */
				/* XXX: There might be some way to check the consequences of all options to determine the best resolve */
				let mut explicit_required = Vec::<NodeIndex>::new();
				for decision_node in pending_decision_nodes {
					let mut selections = Vec::<(NodeIndex, NodeIndex)>::new();

					for decision_option in self.dep_graph.graph.edges_directed(decision_node, Outgoing) {
						if let NodeData::Stub(name) | NodeData::Candidate(name, _) = &self.dep_graph.graph[decision_option.target()] {
							if self.decisions.contains(name) {
								selections.push((decision_node,decision_option.target()));
								continue;
							}
						}

						for existing_requirement in self.dep_graph.graph.edges_directed(decision_option.target(), Incoming) {
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
							self.dep_graph.graph.add_edge(s.0, s.1, EdgeData::Selected);
						}
					}
				}

				if !explicit_required.is_empty() {
					self.is_complete = false;
					return ResolverStatus::DecisionsRequired(
						explicit_required.into_iter().map(|i| {
							/* XXX: Assumes i is a Decision node */
	
							let decision_parent_node = &self.dep_graph.graph[self.dep_graph.graph.edges_directed(i, petgraph::Incoming).next().expect("floating decision node").source()];
							let source = if let NodeData::Stub(name) | NodeData::Candidate(name, _) | NodeData::Virtual(name) = decision_parent_node {
								name.clone()
							} else {
								unimplemented!("decision node attached to incorrect node type")
							};
	
							
							let mut  options = Vec::<String>::new();
							for e in self.dep_graph.graph.edges_directed(i, Outgoing) {
								let target = &self.dep_graph.graph[e.target()];
	
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

	/// Reads the requirements placed on `src` and gets the latest package meeting the requirements.
	/// 
	/// This function makes only finds the latest candidate compatible with the node requirements.
	/// It makes no attempt to resolve conflicts arising from this choice.
	/// # Panics
	/// - If `src` is not a `Candidate` or `Stub`.
	fn determine_package_for_candidate(&mut self, metadb: &crate::metadb::MetaDB, src: NodeIndex) -> Result<(), DeterminePackageError> {
		if let NodeData::Candidate(name, _) | NodeData::Stub(name) = &self.dep_graph.graph[src] {
			let name = name.clone();

			let bounds = self.dep_graph.get_version_bounds_for_node(src).ok_or(DeterminePackageError::VersionBoundsImcompatible)?;

			/* We don't use `bounds` yet, we want to grab every package providing the identifier so we can tell if it exists at all */
			let matching_packages_providing = metadb.get_packages().iter().get_packages_providing(&PackageDescriptor { name: name.clone(), version: VersionBounds::Any });
			if matching_packages_providing.is_empty() {
				Err(DeterminePackageError::IdentifierDoesNotExist)
			} else if matching_packages_providing.len() == 1 {
				/* There's only one package providing so it's a real package */

				let mut m: Vec<_> = matching_packages_providing.into_values().next().expect("next should work when len() == 1").into_iter().mod_version_matches(bounds).collect();
				if m.is_empty() { return Err(DeterminePackageError::NoCompatibleVersion); }
				m = m.into_iter().ksp_version_matches(self.compatible_ksp_versions.clone()).collect();
				if m.is_empty() { return Err(DeterminePackageError::NoCompatibleGameVersion); }

				/* 
				This is the latest package that matches the requirements.
				It's not for this function to determine the later side effects of this decision,
				So we don't need to track the attempted packages or iterate all possible candidates.
				*/

				m.sort();
				let latest = m[0];

				self.dep_graph.set_node_as_package(src, latest);
				
				Ok(())
			} else {
				/* The package is virtual */
				/* We represent the providers as a `Decision` node */
				self.dep_graph.graph[src] = NodeData::Virtual(name);
				let decision = self.dep_graph.graph.add_node(NodeData::Decision);
				self.dep_graph.graph.add_edge(src, decision, EdgeData::Decision);
				for k in matching_packages_providing.keys() {
					let provider = self.dep_graph.get_or_add_node_index(k);
					self.dep_graph.graph.add_edge(decision, provider, EdgeData::Option);
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

	pub fn complete(self) -> Result<PackageTree<Complete>, Box<PackageTree<InProgress>>> {
		if self.is_complete {
			Ok(
				PackageTree {
					decisions: self.decisions,
					compatible_ksp_versions: self.compatible_ksp_versions,
					dep_graph: self.dep_graph,
					compatible_candidates: self.compatible_candidates,
					is_complete: self.is_complete,
					state: std::marker::PhantomData,
				}
			)
		} else {
			Err(Box::new(self))
		}
	}
}

impl PackageTree<Complete> {
	pub fn get_all_packages(&self) -> Vec<crate::metadb::package::PackageIdentifier> {
		dbg!(&self.dep_graph.graph);
		self.dep_graph.graph
			.node_references()
			.filter(|n| matches!(n.1, NodeData::Candidate(_,_)))
			.map(|candidate| {
				let data = candidate.1;
				/* XXX: Assume candidate is a `Candidate` node */
				if let NodeData::Candidate(_, data) = data {
					data.id.clone()
				} else {
					/* Should never actually happen we just can't prove it to the compiler */
					unimplemented!("bad node variant");
				}
			})
			.collect()
		/* TODO: Better install ordering */
	}

	pub fn alter_package_requirements(mut self, add: impl IntoIterator<Item = InstallTarget>, remove: impl IntoIterator<Item = InstallTarget>) -> PackageTree<InProgress> {
		for target in add {
			let new = self.dep_graph.get_or_add_node_index(&target.identifier);
			self.dep_graph.graph.add_edge(self.dep_graph.meta_node, new, EdgeData::Depends(target.required_version.clone()));
		}

		for target in remove {
			todo!();
		}

		PackageTree::<InProgress> {
			decisions: self.decisions,
			compatible_ksp_versions: self.compatible_ksp_versions,
			dep_graph: self.dep_graph,
			compatible_candidates: self.compatible_candidates,
			is_complete: false,
			state: Default::default(),
		}
	}
}

impl<State> PackageTree<State> {
	/// Creates a new `RelationshipResolver`.
	/// 
	/// # Parameters
	/// - `existing_graph`: if adding to a completed resolve, the completed graph can be passed here to shorten the resolve process.
	/// - `compatible_ksp_versions`: Packages for these versions of the game can be installed.
	pub fn new(compatible_ksp_versions: impl IntoIterator<Item = KspVersionReal>) -> PackageTree<Complete> {
		PackageTree {
			decisions: Default::default(),
			compatible_ksp_versions: compatible_ksp_versions.into_iter().collect(),
			dep_graph: Default::default(),
			compatible_candidates: Default::default(),
			is_complete: true,
			state: std::marker::PhantomData,
		}
	}

	pub fn clear_all_packages(&mut self) {
		self.dep_graph.clear_all_packages();
	}

	pub fn clear_loose_packages(&mut self) {
		self.dep_graph.clear_loose_nodes();
	}
}