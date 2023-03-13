use ckan_rs::game_instance::GameInstance;

#[tokio::test]
async fn full_install() {
	use ckan_rs::relationship_resolver::*;
	use ckan_rs::metadb::ckan::*;

	let options = ckan_rs::CkanRsOptions::default();
	
	let db = {
		if let Ok(db) = ckan_rs::MetaDB::load_from_disk(&options) {
			db
		} else {
			let db = tokio::task::spawn_blocking(ckan_rs::metadb::generate_latest).await.expect("failed to join task.").expect("failed to generate metadb.");
			db.save_to_disk(&options).expect("failed to save metadb.");
			db
		}
	};

	let compatible_ksp_versions = vec![KspVersion::new("1.12"), KspVersion::new("1.11")];
	
	let requirements = vec![
		InstallRequirement {identifier: "ModuleManager".to_string(), ..Default::default() },
		// InstallRequirement {identifier: "MechJeb2".to_string(), ..Default::default() }, /* 404s? my fault? */
		// InstallRequirement {identifier: "ProceduralParts".to_string(), ..Default::default() },
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

	let mut instance = {
		let instance_path = ckan_rs_test_utils::create_fake_game_instance().expect("failed to create test game instance.");
		GameInstance::new(&instance_path, instance_path.parent().unwrap().join("deployment")).unwrap()
	};
	
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
		ckan_rs::installer::content::extract_content_to_deployment(&options, &instance, package).unwrap();
		instance.enable_package(package);
		dbg!(&package.install);
	}

	ckan_rs::installer::deployment::redeploy_packages(db, &mut instance).await.expect("deployment failed");

}