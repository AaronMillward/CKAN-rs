#[test]
fn resolve_dependency() {
	use ckan_rs::relationship_resolver::*;
	use ckan_rs::metadb::ckan::*;
	
	let db = {
		let p = env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/metadb.bin";
		eprintln!("reading db from {}", p);
		ckan_rs_test_utils::get_metadb(Some(std::path::PathBuf::from(p)))
		// ckan_rs_test_utils::get_metadb(None)
	};

	let compatible_ksp_versions = vec![KspVersion::new("1.12"), KspVersion::new("1.11")];
	
	let requirements = vec![
		InstallRequirement {identifier: "MechJeb2".to_string(), ..Default::default() },
		InstallRequirement {identifier: "ProceduralParts".to_string(), ..Default::default() },
		InstallRequirement {identifier: "KSPInterstellarExtended".to_string(), ..Default::default() },
		InstallRequirement {identifier: "Parallax".to_string(), ..Default::default() },
	];

	let mut resolver = ResolverBuilder::new(&db)
		.add_package_requirements(requirements)
		.compatible_ksp_versions(compatible_ksp_versions)
		.build();

	loop {
		match resolver.attempt_resolve() {
			ResolverStatus::Complete => {
				eprintln!("Final Package List:");
				for package in resolver.finalize().expect("resolver complete status but not complete flagged").get_new_packages() {
					eprintln!("\tID: {} VERSION: {:?}", package.identifier, package.version);
				}
				break;
			},
			ResolverStatus::DecisionsRequired(infos) => {
				for info in infos {
					let mut options = info.options.clone();
					options.sort(); /* We want to always choses the same option */
					eprintln!("choosing `{}` from options {:?} required by `{}`", &info.options[0], &info.options, info.source);
					resolver.add_decision(&info.options[0]);
				}
			},
			ResolverStatus::Failed(fails) => {
				for fail in fails {
					eprintln!("Package `{}` failed with {:?}", fail.0, fail.1)
				}
				eprintln!("-- BEGIN FAILED GRAPH DUMP --");
				dbg!(resolver.get_graph()); 
				eprintln!("-- END FAILED GRAPH DUMP --");
				panic!("resolver failed"); 
			},
		}
	}

}