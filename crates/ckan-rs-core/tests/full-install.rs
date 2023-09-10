#[tokio::test]
async fn full_install() {
	use ckan_rs::game_instance::GameInstance;
	use ckan_rs::relationship_resolver::*;
	use ckan_rs::metadb::package::*;

	env_logger::builder().is_test(true).try_init().expect("failed to create logger.");

	let config = ckan_rs::CkanRsConfig::default();
	
	let db = {
		if let Ok(db) = ckan_rs::MetaDB::load_from_disk(&config) {
			db
		} else {
			let db = ckan_rs::metadb::generate_latest().await.expect("failed to generate metadb.");
			db.save_to_disk(&config).expect("failed to save metadb.");
			db
		}
	};

	let compatible_ksp_versions = vec![KspVersionReal::new("1.12").expect("failed to create version from string."), KspVersionReal::new("1.11").expect("failed to create version from string.")];
	
	let requirements = vec![
		InstallTarget {identifier: "ModuleManager".to_string(), ..Default::default() },
		// InstallRequirement {identifier: "MechJeb2".to_string(), ..Default::default() }, /* 404s? my fault? */
		InstallTarget {identifier: "ProceduralParts".to_string(), ..Default::default() },
		InstallTarget {identifier: "KSPInterstellarExtended".to_string(), ..Default::default() },
	];


	let mut resolver = ResolverBuilder::new(&db)
		.compatible_ksp_versions(compatible_ksp_versions.clone())
		.add_package_requirements(requirements)
		.build();
		
	let packages = loop {
		match resolver.attempt_resolve() {
			ResolverStatus::Complete => {
				break resolver.finalize().expect("resolve not complete.").get_new_packages();
			},
			ResolverStatus::DecisionsRequired(infos) => {
				for info in infos {
					let mut options = info.options.clone();
					options.sort(); /* We want to always choses the same option */
					log::info!("choosing `{}` from options {:?} required by `{}`", &info.options[0], &info.options, info.source);
					resolver.add_decision(&info.options[0]);
				}
			},
			ResolverStatus::Failed(fails) => {
				log::error!("RESOLVER FAILS DUMP\n{:?}", fails);
				panic!("resolver failed."); 
			},
		}
	};

	log::info!("Final Package List:");
	for package in &packages {
		log::info!("\tID: {} VERSION: {:?}", package.identifier, package.version);
	}

	let mut instance = {
		let instance_path = ckan_rs_test_utils::create_fake_game_instance().expect("failed to create test game instance.");
		GameInstance::new(&config, db.get_game_builds(), "test".into(), &instance_path, instance_path.parent().unwrap().join("deployment")).unwrap()
	};
	
	instance.set_compatible_ksp_versions(compatible_ksp_versions);

	let packages = packages.iter()
		.map(|m| db.get_from_unique_id(m).expect("metadb package not found"))
		.collect::<Vec<_>>();

	{
		let download_results = ckan_rs::installation::download::download_packages_content(&config, packages.as_slice(), false).await;
		for result in download_results {
			if result.1.is_err() { panic!("failed to download package {} {:?}", result.0.identifier.identifier, result.1)}
		}
	}

	for package in packages {
		ckan_rs::installation::content::extract_content_to_deployment(&config, &instance, package).unwrap();
		instance.enable_package(package);
	}

	instance.redeploy_packages(&db).await.expect("deployment failed");
}