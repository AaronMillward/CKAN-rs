#[test]
fn resolve_dependency() {
	use ckan_rs::modulemanager::dependency_resolver::*;
	use ckan_rs::metadb::ckan::*;
	
	let db = {
		let p = env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/metadb.bin";
		eprintln!("reading db from {}", p);
		ckan_rs_test_utils::get_metadb(Some(std::path::PathBuf::from(p)))
		// ckan_rs_test_utils::get_metadb(None)
	};

	let compatible_ksp_versions = vec![KspVersion::new("1.12")];
	
	let requirements = vec![
		InstallRequirement {mod_identifier: "MechJeb2".to_string(), ..Default::default() },
		InstallRequirement {mod_identifier: "ProceduralParts".to_string(), ..Default::default() },
		InstallRequirement {mod_identifier: "KSPInterstellarExtended".to_string(), ..Default::default() },
		InstallRequirement {mod_identifier: "Parallax".to_string(), ..Default::default() },
	];

	let mut resolver = RelationshipResolver::new(&db, &requirements, None, compatible_ksp_versions);

	loop {
		match resolver.attempt_resolve() {
			ResolverStatus::Complete => { dbg!(resolver.get_complete_graph().unwrap()); break; },
			ResolverStatus::DecisionsRequired(infos) => {
				for info in infos {
					resolver.add_decision(&info.options[0]);
				}
			},
			ResolverStatus::Failed => { panic!("resolver failed"); },
		}
	}

	// eprintln!("Final Module List:");
	// for m in resolver.get_final_module_list().unwrap() {
	// 	eprintln!("\tID: {} VERSION: {:?}", m.unique_id.identifier, m.unique_id.version);
	// }

}