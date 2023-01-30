use ckan_rs::game_instance::GameInstance;

#[test]
fn full_install() {
	use ckan_rs::manager::relationship_resolver::*;
	use ckan_rs::metadb::ckan::*;

	let options = ckan_rs::CkanRsOptions::default();
	
	let db = {
		let p = env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/metadb.bin";
		eprintln!("reading db from {}", p);
		ckan_rs_test_utils::get_metadb(Some(std::path::PathBuf::from(p)))
		// ckan_rs_test_utils::get_metadb(None)
	};

	let compatible_ksp_versions = vec![KspVersion::new("1.12"), KspVersion::new("1.11")];
	
	let requirements = vec![
		InstallRequirement {mod_identifier: "MechJeb2".to_string(), ..Default::default() },
		InstallRequirement {mod_identifier: "ProceduralParts".to_string(), ..Default::default() },
		InstallRequirement {mod_identifier: "KSPInterstellarExtended".to_string(), ..Default::default() },
		InstallRequirement {mod_identifier: "Parallax".to_string(), ..Default::default() },
	];

	let mut resolver = RelationshipResolver::new(&db, &requirements, None, compatible_ksp_versions);

	let modules = loop {
		match resolver.attempt_resolve() {
			ResolverStatus::Complete(new_modules) => {
				break new_modules;
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
				panic!("resolver failed"); 
			},
		}
	};

	eprintln!("Final Module List:");
	for m in modules {
		eprintln!("\tID: {} VERSION: {:?}", m.identifier, m.version);
	}

	let mut instance = GameInstance::new(env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/fake-game-dir").unwrap();

	instance.compatible_ksp_versions = compatible_ksp_versions;

	let client = reqwest::Client::builder()
		.https_only(options.https_only())
		.build()
		.unwrap();

	ckan_rs::installer::download::download_modules_content(&options, &client, modules).await;

	for module in modules {
		ckan_rs::installer::content::extract_content_to_deployment(&options, module).unwrap();
	}

	ckan_rs::installer::deployment::redeploy_modules(&options, db, &mut instance);

}