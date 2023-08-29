#[tokio::test]
async fn resolve_dependency() {
	use ckan_rs::relationship_resolver::*;
	use ckan_rs::metadb::package::*;
	
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
		InstallTarget {identifier: "MechJeb2".to_string(), required_version: PackageVersionBounds::Explicit(PackageVersion::new("0:2.12.0.0").expect("failed to create version string.")) },
		InstallTarget {identifier: "ProceduralParts".to_string(), required_version: PackageVersionBounds::Explicit(PackageVersion::new("0:v2.2.0").expect("failed to create version string.")) },
		InstallTarget {identifier: "KSPInterstellarExtended".to_string(), required_version: PackageVersionBounds::Explicit(PackageVersion::new("0:1.26.5").expect("failed to create version string.")) },
		// InstallRequirement {identifier: "Parallax".to_string(), ..Default::default() },
	];

	let mut resolver = ResolverBuilder::new(&db)
		.add_package_requirements(requirements)
		.compatible_ksp_versions(compatible_ksp_versions)
		.build();

	loop {
		match resolver.attempt_resolve() {
			ResolverStatus::Complete => {
				let packages = resolver.finalize().expect("resolver complete status but not complete flagged").get_new_packages();

				/* XXX: Potential false positive if the dependency list of these mods changes. */
				let expected = [
					// "Parallax",
					// "Kopernicus",
					// "Parallax-Textures",
					// "Parallax-StockTextures",
					// "Parallax-Scatter-Textures",

					"KSPInterstellarExtended",
					"InterstellarFuelSwitch-Core",
					"TweakScale",
					"ModuleManager",

					"ProceduralParts",
					"ModuleManager",

					"MechJeb2",
					"ModuleManager",
				];

				let mut missing = Vec::<String>::new();

				for id in expected {
					if !packages.iter().any(|p| p.identifier == id) {
						missing.push(id.to_string());
					}
				}

				if !missing.is_empty() {
					panic!("missing expected package(s) {:?}", missing)
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
				panic!("resolver failed"); 
			},
		}
	}
}