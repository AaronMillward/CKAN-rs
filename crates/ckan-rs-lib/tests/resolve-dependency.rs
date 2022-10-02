#[test]
fn resolve_dependency() {
	use std::collections::HashSet;
	use ckan_rs::modulemanager::dependencyresolver::*;
	use ckan_rs::metadb::ckan::*;
	
	let db = ckan_rs_test_utils::get_metadb();
	let compatible_ksp_versions = HashSet::<KspVersion>::new();
	let requirements = vec![InstallRequirement {mod_identifier: "MechJeb2".to_string(), ..Default::default() } ];

	let resolver = RelationshipResolver::new(compatible_ksp_versions, requirements, &db);

	loop {
		/* TODO: Requires redesign of RelationshipResolver */
		let mut process = resolver.step();

	}

}