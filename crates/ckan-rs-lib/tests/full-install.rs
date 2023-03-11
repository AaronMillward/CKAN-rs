use ckan_rs::game_instance::GameInstance;

#[tokio::test]
async fn full_install() {
	use ckan_rs::relationship_resolver::*;
	use ckan_rs::metadb::ckan::*;

	let options = ckan_rs::CkanRsOptions::default();
	
	let db: ckan_rs::MetaDB = {
		let p = env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/metadb.bin";
		eprintln!("reading db from {}", p);
		ckan_rs_test_utils::get_metadb(Some(std::path::PathBuf::from(p)))
		// tokio::task::spawn_blocking(|| ckan_rs_test_utils::get_metadb(None)).await.expect("failed to generate metadb.")
	};

	let compatible_ksp_versions = vec![KspVersion::new("1.12"), KspVersion::new("1.11")];
	
	let requirements = vec![
		// InstallRequirement {identifier: "MechJeb2".to_string(), ..Default::default() }, /* 404s? my fault? */
		InstallRequirement {identifier: "ProceduralParts".to_string(), ..Default::default() },
	];

	let mut resolver = RelationshipResolver::new(&db, &requirements, None, compatible_ksp_versions.clone());

	let packages = loop {
		match resolver.attempt_resolve() {
			ResolverStatus::Complete(new_packages) => {
				break new_packages;
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
				eprintln!("RESOLVER FAILS DUMP\n{:?}", fails);
				panic!("resolver failed."); 
			},
		}
	};

	eprintln!("Final Package List:");
	for package in &packages {
		eprintln!("\tID: {} VERSION: {:?}", package.identifier, package.version);
	}

	let mut instance = GameInstance::new(env!("CARGO_MANIFEST_DIR").to_owned() + "/test-data/fake-game-dir").unwrap();

	instance.compatible_ksp_versions = compatible_ksp_versions;

	let client = reqwest::Client::builder()
		.https_only(options.https_only())
		.build()
		.unwrap();

	let packages = packages.iter()
		.map(|m| db.get_from_unique_id(m).expect("metadb package not found"))
		.collect::<Vec<_>>();

	{
		let download_results = ckan_rs::installer::download::download_packages_content(&options, &client, packages.as_slice(), false).await;
		for result in download_results {
			if result.1.is_err() { panic!("failed to download package {} {:?}", result.0.identifier.identifier, result.1)}
		}
	}

	for package in packages {
		ckan_rs::installer::content::extract_content_to_deployment(&options, package).unwrap();
		instance.add_enabled_packages(package);
		dbg!(&package.install);
	}

	ckan_rs::installer::deployment::redeploy_packages(&options, db, &mut instance).await.expect("deployment failed");

}