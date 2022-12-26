// #[test]
// fn full_install() {
// 	use ckan_rs::manager::relationship_resolver::*;
// 	use ckan_rs::metadb::ckan::*;
	
// 	let db = {
// 		let p = env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/metadb.bin";
// 		eprintln!("reading db from {}", p);
// 		ckan_rs_test_utils::get_metadb(Some(std::path::PathBuf::from(p)))
// 		// ckan_rs_test_utils::get_metadb(None)
// 	};

// 	let compatible_ksp_versions = vec![KspVersion::new("1.12"), KspVersion::new("1.11")];
	
// 	let requirements = vec![
// 		InstallRequirement {mod_identifier: "MechJeb2".to_string(), ..Default::default() },
// 		InstallRequirement {mod_identifier: "ProceduralParts".to_string(), ..Default::default() },
// 		InstallRequirement {mod_identifier: "KSPInterstellarExtended".to_string(), ..Default::default() },
// 		InstallRequirement {mod_identifier: "Parallax".to_string(), ..Default::default() },
// 	];

// 	let mut resolver = RelationshipResolver::new(&db, &requirements, None, compatible_ksp_versions);

// 	loop {
// 		match resolver.attempt_resolve() {
// 			ResolverStatus::Complete(new_modules) => {
// 				dbg!(resolver.get_complete_graph().unwrap());
// 				eprintln!("Final Module List:");
// 				for m in new_modules {
// 					eprintln!("\tID: {} VERSION: {:?}", m.identifier, m.version);
// 				}
// 				break;
// 			},
// 			ResolverStatus::DecisionsRequired(infos) => {
// 				for info in infos {
// 					let mut options = info.options.clone();
// 					options.sort(); /* We want to always choses the same option */
// 					eprintln!("choosing `{}` from options {:?} required by `{}`", &info.options[0], &info.options, info.source);
// 					resolver.add_decision(&info.options[0]);
// 				}
// 			},
// 			ResolverStatus::Failed(fails) => {
// 				for fail in fails {
// 					eprintln!("Module `{}` failed with {:?}", fail.0, fail.1)
// 				}
// 				eprintln!("-- BEGIN FAILED GRAPH DUMP --");
// 				dbg!(resolver.get_graph()); 
// 				eprintln!("-- END FAILED GRAPH DUMP --");
// 				panic!("resolver failed"); 
// 			},
// 		}
// 	}

// }