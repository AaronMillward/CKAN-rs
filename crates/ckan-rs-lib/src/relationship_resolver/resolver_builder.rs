use super::ResolverProcessor;
use super::InstallRequirement;
use super::DependencyGraph;
use super::dependency_graph::NodeData;
use super::dependency_graph::EdgeData;
use crate::metadb::*;
use crate::metadb::package::*;

pub struct ResolverBuilder<'db> {
	dep_graph: Option<DependencyGraph>,
	metadb: &'db MetaDB,
	compatible_ksp_versions: Vec<KspVersionReal>,

	requirements: Vec<InstallRequirement>,
}

impl<'db> ResolverBuilder<'db> {
	pub fn new(metadb: &'db MetaDB) -> Self {
		Self {
			metadb,
			dep_graph: None,
			compatible_ksp_versions: Default::default(),
			requirements: Default::default(),
		}
	}

	pub fn add_package_requirements(mut self, requirements: impl IntoIterator<Item = InstallRequirement>) -> Self {
		for requirement in requirements {
			self.requirements.push(requirement);
		}
		self
	}

	pub fn existing_graph(mut self, graph: DependencyGraph) -> Self {
		self.dep_graph = Some(graph);
		self
	}

	pub fn compatible_ksp_versions(mut self, ksp_versions: impl IntoIterator<Item = KspVersionReal>) -> Self {
		self.compatible_ksp_versions = ksp_versions.into_iter().collect();
		self
	}

	pub fn build(self) -> ResolverProcessor<'db> {
		let (mut graph, meta_node) = if let Some(graph) = self.dep_graph {
			let meta = graph.node_weights()
				.enumerate()
				.find(|(_, data)| { matches!(data, NodeData::Meta) })
				.map(|(i,_)| petgraph::graph::node_index(i))
				.expect("existing graph missing meta node.");
			(graph, meta)
		} else {
			let mut graph = DependencyGraph::default();
			let meta = graph.add_node(NodeData::Meta);
			(graph, meta)
		};

		for m in self.requirements {
			let new = super::dependency_graph::get_or_add_node_index(&mut graph, &m.identifier);
			graph.add_edge(meta_node, new, EdgeData::Depends(m.required_version.clone()));
		}

		ResolverProcessor::new(self.metadb, graph, meta_node, self.compatible_ksp_versions)
	}
}