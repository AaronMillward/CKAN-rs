use super::*;

pub enum TransactionStatus<'db> {
	Ok(GameInstance),
	DecisionsRequired(GameInstanceTransaction<'db>, Vec<relationship_resolver::DecisionInfo>),
	Failed(GameInstance, Vec<(String, relationship_resolver::DetermineModuleError)>),
}

pub struct GameInstanceTransaction<'db> {
	decisions: Vec<String>,

	metadb: &'db MetaDB,

	inner: GameInstance,
}

impl<'db> GameInstanceTransaction<'db> {
	pub fn new(profile: GameInstance, metadb: &'db MetaDB) -> GameInstanceTransaction {
		Self {
			inner: profile,
			decisions: Default::default(),
			metadb,
		}
	}

	pub fn add_decision(&mut self, identifier: &str) {
		self.decisions.push(identifier.to_owned());
	}

	pub fn commit(self) -> TransactionStatus<'db> {
		/* TODO: Less brute force approach */
		let mut resolver = relationship_resolver::RelationshipResolver::new(self.metadb, &self.inner.wanted, None, self.inner.compatible_ksp_versions.clone());
		for d in &self.decisions {
			resolver.add_decision(d);
		}

		match resolver.attempt_resolve() {
			relationship_resolver::ResolverStatus::Complete(mods) => {
				/* TODO: Install Modules */
				TransactionStatus::Ok(self.inner)
			},
			relationship_resolver::ResolverStatus::DecisionsRequired(decs) => {
				TransactionStatus::DecisionsRequired(self, decs)
			},
			relationship_resolver::ResolverStatus::Failed(err) => {
				TransactionStatus::Failed(self.inner, err)
			},
		}
	}

	pub fn cancel(self) -> GameInstance {
		self.inner
	}
}