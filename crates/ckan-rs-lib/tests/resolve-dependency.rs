#[test]
fn resolve_dependency() {
	use std::collections::HashSet;
	use ckan_rs::modulemanager::dependencyresolver::*;
	use ckan_rs::metadb::ckan::*;
	
	let db = {
		let p = env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/metadb.bin";
		eprintln!("reading db from {}", p);
		ckan_rs_test_utils::get_metadb(Some(std::path::PathBuf::from(p)))
		// ckan_rs_test_utils::get_metadb(None)
	};
	let compatible_ksp_versions = HashSet::<KspVersion>::new();
	let requirements = vec![
		InstallRequirement {mod_identifier: "MechJeb2".to_string(), ..Default::default() },
		InstallRequirement {mod_identifier: "ProceduralParts".to_string(), ..Default::default() },
	];

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

	eprintln!("Final Module List:");
	for m in resolver.get_final_module_list().unwrap() {
		eprintln!("\tID: {} VERSION: {:?}", m.identifier, m.version);
	}

}