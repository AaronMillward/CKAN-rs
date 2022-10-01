//!

use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

use rusqlite::params;

use crate::metadb::ckan::*;
use crate::metadb::MetaDB;

#[derive(Debug, Default)]
pub struct InstallRequirement {
	mod_identifier: String,
	required_version: Option<ModVersion>
}

/// Describes a decision to be made by the user when mutiple providers are available for a given module
#[derive(Debug)]
pub struct RelationshipMutlipleProvidersDecision<'a> {
	inner: RelationshipResolver<'a>,
	virtual_identifier: String,
	options: HashSet<String>,
	selection: String,
}

impl<'a> RelationshipMutlipleProvidersDecision<'a> {
	pub fn get_options(&self) -> &HashSet<String> {
		&self.options
	}

	pub fn select(&mut self, choice: String) -> bool {
		if !self.options.contains(&choice) {
			false
		} else {
			self.selection = choice;
			true
		}
	}

	pub fn confirm(mut self) -> RelationshipResolver<'a> {
		self.inner.add_decision(self.virtual_identifier, self.selection);
		return self.inner
	}
}

#[derive(Debug)]
pub enum RelationshipProcess<'a> {
	MultipleProviders(RelationshipMutlipleProvidersDecision<'a>),
	Complete(Vec<super::InstalledModule>),
}

/// DependencyResolver will take a list of top level requirements and generate a list of required modules
#[derive(Debug)]
pub struct RelationshipResolver<'db> {
	metadb: &'db MetaDB,
	requirements: Vec<InstallRequirement>,
	compatible_ksp_versions: HashSet<String>,
	decisions: HashMap<String, String>,
	incomplete: HashMap<String, ()>,
	queue: VecDeque<String>,
	completed: HashSet<String>,
	modules: Vec<super::InstalledModule>,
}

impl<'db> RelationshipResolver<'db> {
	pub fn new<R>(
		compatible_ksp_versions: HashSet<String>,
		requirements: Vec<InstallRequirement>,
		metadb: &'db MetaDB
	) -> Self {
		Self {
			metadb,
			requirements,
			compatible_ksp_versions,
			decisions: Default::default(),
			incomplete: Default::default(),
			queue: Default::default(),
			completed: Default::default(),
			modules: Default::default(),
		}
	}

	/// runs the resolver until complete or a decision has to be made
	pub fn process(self) -> RelationshipProcess<'db> {
		/* 
		- Select depends, conflicts, ksp_version for each mod identifier
		- if requirement has a specific version limit select to that version
		- if there are no matching versions we have an error
		- recurse until no new deps are found
		*/

		/* We use a breadth first approach as I think users will prefer to make higher level decisions first */

		let mut get_identifier = self.metadb.get_connection().prepare(
			"SELECT id, virtual FROM identifier WHERE name = ?1"
		).expect("Failed to compile statement");
		let mut get_mods_providing = self.metadb.get_connection().prepare(
			"SELECT name FROM identifier WHERE id = (SELECT DISTINCT indentifier_id FROM mod_provides WHERE mod_id = (SELECT mod_id FROM mod_provides WHERE identifier_id = ?1))"
		).expect("Failed to compile statement");

		let mut get_mod = self.metadb.get_connection().prepare(
			"SELECT ksp_version,  FROM identifier WHERE id = (SELECT DISTINCT indentifier_id FROM mod_provides WHERE mod_id = (SELECT mod_id FROM mod_provides WHERE identifier_id = ?1))"
		).expect("Failed to compile statement");

		loop {
			/* Pop next in queue */
			/* If it is a virtual identifier ask which option */
			/* add deps of the module to the queue if they aren't in completed already */
			/* repeat step 1 until no queue is empty */

			let mut identifier = {
				let opt = self.queue.pop_front();
				if opt.is_none() {
					break;
				}
				opt.unwrap()
			};

			if self.completed.contains(&identifier) {
				continue;
			}

			/* Check if virtual */ {
				let (identifier_id, is_virtual): (u64, bool) = get_identifier.query_row(
					params![identifier],
					|r| { Ok((r.get_unwrap(0), r.get_unwrap(1))) }
				).expect("identifier does not exist");

				if is_virtual {
					/* TODO: add install requirements to decisions to avoid asking when already chosen */
					if !self.decisions.contains_key(&identifier) {
						let providers = get_mods_providing.query_map(
							params![identifier_id],
							|r| { r.get(0) }
						).unwrap().map(|r| r.unwrap()).collect::<HashSet<String>>();
	
						return RelationshipProcess::MultipleProviders(
							RelationshipMutlipleProvidersDecision {
								inner: self,
								virtual_identifier: identifier,
								options: providers,
								selection: "".to_string(),
							}
						)
					} else {
						identifier = self.decisions.get(&identifier).unwrap().clone();
					}
				}
			}
			/* We now have a real identifier */

			
			
			/* Filter to only matching game versions */


		}
		todo!()
	}

	fn add_decision(&mut self, virtual_identifier: String, identifier: String) {
		self.decisions.insert(virtual_identifier, identifier);
	}
}