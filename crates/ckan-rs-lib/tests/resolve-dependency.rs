#[test]
fn resolve_dependency() {
	use std::collections::HashSet;
	use ckan_rs::modulemanager::dependencyresolver::*;
	use ckan_rs::metadb::ckan::*;
	
	let db = ckan_rs_test_utils::get_metadb();
	let compatible_ksp_versions = HashSet::<KspVersion>::new();
	let requirements = vec![InstallRequirement {mod_identifier: "MechJeb2".to_string(), ..Default::default() } ];

	let mut resolver = RelationshipResolver::new(compatible_ksp_versions, requirements, &db);

	loop {
		let process = resolver.step();
		match process {
			RelationshipProcess::Incomplete => {},
			RelationshipProcess::MultipleProviders(mut decision) => {
				decision.select(decision.get_options().iter().next().cloned().unwrap());
				resolver.add_decision(decision);
			},
			RelationshipProcess::Halt => {
				dbg!(resolver.get_failed_resolves());
				panic!("Resolver Halted");
			},
			RelationshipProcess::Complete => { break; },
		}
	}
}