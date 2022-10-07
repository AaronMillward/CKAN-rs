#[test]
fn resolve_dependency() {
	use ckan_rs::modulemanager::dependencyresolver::*;
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

	let mut resolver = RelationshipResolver::new(compatible_ksp_versions, requirements, &db);

	loop {
		let process = resolver.step();
		match process {
			RelationshipProcess::Incomplete => {},
			RelationshipProcess::MultipleProviders(mut decision) => {
				let mut options = decision.get_options().iter().collect::<Vec<_>>();
				options.sort(); /* Sort to get a consistent result when testing */
				let dec = options[0].clone();
				eprintln!("Adding \"{}\" to decisions from list {:?}", dec, decision.get_options());
				decision.select(dec);
				resolver.add_decision(decision);
			},
			RelationshipProcess::Halt => {
				eprintln!("Resolver halted, printing failures:");
				for fail in resolver.get_failed_resolves() {
					match fail {
						FailedResolve::ModulesConflict(l, r) => eprintln!("Conflict\n\t{:?}\n\t\t{:?}\n\t\t{:?}\n\t{:?}\n\t\t{:?}\n\t\t{:?}", &l.identifier, &l.version, &l.conflicts, &r.identifier, &r.version, &r.conflicts),
						f => eprintln!("{:?}", f),
					}
				}
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