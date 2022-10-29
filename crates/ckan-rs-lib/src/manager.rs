use crate::metadb::ckan;
use crate::metadb::MetaDB;

pub mod relationship_resolver;

/* TODO: Install Reason */

pub enum TransactionStatus<'db> {
	Ok(Profile),
	DecisionsRequired(ProfileTransaction<'db>, Vec<relationship_resolver::DecisionInfo>),
	Failed(Profile, Vec<(String, relationship_resolver::DetermineModuleError)>),
}

pub struct ProfileTransaction<'db> {
	decisions: Vec<String>,

	metadb: &'db MetaDB,

	inner: Profile,
}

impl<'db> ProfileTransaction<'db> {
	pub fn new(profile: Profile, metadb: &'db MetaDB) -> ProfileTransaction {
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

	pub fn cancel(self) -> Profile {
		self.inner
	}
}

pub struct Profile {
	pub compatible_ksp_versions: Vec<ckan::KspVersion>,
	wanted: Vec<relationship_resolver::InstallRequirement>,
}

impl Profile {
	pub fn start_transaction(self, metadb: &MetaDB) -> ProfileTransaction {
		ProfileTransaction::new(self, metadb)
	}
}