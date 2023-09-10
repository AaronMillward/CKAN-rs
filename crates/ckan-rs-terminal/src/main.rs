use std::io::Write;

#[tokio::main]
async fn main() {
	env_logger::init();

	let mut opts;
	
	/* Parse console input */ 
	let parsed_options = {
		let args: Vec<String> = std::env::args().collect();
	
		opts = getopts::Options::new();
		opts.optflag( "h", "help",       "Show help");
		opts.optflag( "v", "verbose",    "Increased vebosity");
		opts.parsing_style(getopts::ParsingStyle::FloatingFrees);
	
		let parsed_options = match opts.parse(&args[1..]) {
			Ok(m)  => { m }
			Err(e) => { println!("Unable to parse options: {}", e); return }
		};
		
		if parsed_options.opt_present("h") {
			eprintln!("{}", opts.usage(""));
			return;
		}

		parsed_options
	};

	let config = ckan_rs_core::Config::load_from_disk().unwrap_or_else(|e| {
		log::warn!("Failed to read config file: {}", e);
		log::warn!("Using default config.");
		ckan_rs_core::Config::default()
	});

	async fn genreate_and_save_new_metadb(config: &ckan_rs_core::Config) -> ckan_rs_core::Result<ckan_rs_core::MetaDB> {
		let db = ckan_rs_core::metadb::generate_latest().await?;
		db.save_to_disk(config)?;
		Ok(db)
	}

	let db = match ckan_rs_core::MetaDB::load_from_disk(&config) {
		Ok(db) => db,
		Err(e) => {
			match e {
				ckan_rs_core::Error::IO(e) => {
					match e.kind() {
						std::io::ErrorKind::NotFound => {
							let res = genreate_and_save_new_metadb(&config).await;
							match res {
								Ok(db) => db, 
								Err(e) => {
									log::error!("Failed to generate and save metadb: {}", e);
									return
								}
							}
						}
						_ => {
							log::error!("Failed to open MetaDB due to IO error: {}", e);
							return;
						}
					}
				},
				ckan_rs_core::Error::Parse(_) => {
					log::warn!("Failed to open MetaDB due to parsing error, DB format likely changed. regenerating...");
					let res = genreate_and_save_new_metadb(&config).await;
					match res {
						Ok(db) => db, 
						Err(_) => {
							log::error!("Failed to generate metadb");
							return
						}
					}
				},
				_ => unimplemented!("unexpected error type."),
			}
		}
	};

	if parsed_options.free.is_empty() {
		/* TODO: Error message */
		return;
	}

	if parsed_options.free.get(0).unwrap() == "instance" {
		if parsed_options.free.get(1).unwrap() == "create" {
			let name = match parsed_options.free.get(2) {
				Some(p) => p,
				None => { log::error!("Instance name not provided."); return },
			};
		
			let instance_path = match parsed_options.free.get(3) {
				Some(p) => std::path::PathBuf::from(p),
				None => { log::error!("Instance path not provided."); return },
			};
			
			match create_instance(&config, &db, instance_path, name) {
				Ok(_) => {},
				Err(e) => log::info!("Failed to create instance due to error: {:?}", e),
			}
		}
	} else if parsed_options.free.get(0).unwrap() == "install" {
		let name = match parsed_options.free.get(1) {
			Some(p) => p,
			None => { log::error!("Instance name not provided."); return },
		};

		let package_names = &parsed_options.free[2..];

		match install_packages(&config, &db, name, package_names).await {
			Ok(_) => {},
			Err(e) => log::info!("Failed to install packages due to error: {:?}", e),
		}
	}
}

fn create_instance(config: &ckan_rs_core::Config, db: &ckan_rs_core::MetaDB, instance_path: impl AsRef<std::path::Path>, name: impl AsRef<str>) -> Result<(), Error> {
	log::trace!("Attempting to create new instance");

	let instance_path = instance_path.as_ref();

	let deployment_dir = {
		let base = instance_path.parent().expect("Instance should have a parent directory.");
		let instance_dir_name = instance_path.file_name().expect("instance path should not terminate in \"..\"");
		base.join(instance_dir_name.to_str().expect("instance directory name should be valid unicode.").to_owned() + "-deployment")
	};

	let instance = match ckan_rs_core::game_instance::GameInstance::new(config, db.get_game_builds(), name.as_ref().to_string(), instance_path, deployment_dir) {
		Ok(ins) => ins,
		Err(e) => match e {
			ckan_rs_core::Error::IO(ref inner) => {
				log::error!("Game instance is missing required files. possibly not a valid instance? IO Error: {}", inner);
				return Err(Error::CKANrsError(e));
			},
			ckan_rs_core::Error::Parse(ref inner) => {
				log::error!("Failed to get build id for instance. IO Error: {}", inner);
				return Err(Error::CKANrsError(e));
			},
			ckan_rs_core::Error::AlreadyExists => {
				log::error!("An instance with this name or game root already exists. Error: {}", e);
				return Err(Error::CKANrsError(e));
			}
			_ => unimplemented!("unexpected error kind when creating game instance."),
		},
	};

	log::debug!("Saving new instance to disk.");
	instance.save_to_disk(config)?;

	log::info!("Created new instance succesfully.");
	Ok(())
}

async fn install_packages(config: &ckan_rs_core::Config, db: &ckan_rs_core::MetaDB, instance_name: impl AsRef<str>, package_names: impl IntoIterator<Item = impl AsRef<str>>) -> Result<(), Error> {
	let mut instance = ckan_rs_core::game_instance::GameInstance::load_by_name(config, instance_name)?;

	use ckan_rs_core::relationship_resolver::*;

	let requirements: Vec<_> = package_names.into_iter().map(|s| InstallTarget {identifier: s.as_ref().into(), required_version: ckan_rs_core::metadb::package::VersionBounds::Any}).collect();
	
	let (added, removed) = instance.alter_package_requirements(db, requirements, vec![], |tree, infos| {
		for info in infos {
			let mut options = info.options.clone();
			options.sort();
			println!("Multiple providers of [{}]. select one.", info.source);
			for (i,opt) in options.iter().enumerate() {
				print!("{}) {} ", i, opt);
			}

			let stdin = std::io::stdin();
			loop {
				let mut input = String::new();
				if stdin.read_line(&mut input).is_ok() {
					if let Ok(ans) = input.parse::<usize>() {
						if ans < options.len() {
							tree.add_decision(&info.options[ans]);
							break;
						}
					}
				}
				println!("Input invalid.")
			}
		}
	}).map_err(|_| Error::Resolver)?;

	println!("Resolver complete.");
	println!("Removing packages:");
	for package in &removed {
		println!("\tID: {} VERSION: {:?}", package.identifier, package.version);
	}
	println!("Adding packages:");
	for package in &added {
		println!("\tID: {} VERSION: {:?}", package.identifier, package.version);
	}

	let stdin = std::io::stdin();
	print!("Commit changes? [(y)/n] ");
	let _ = std::io::stdout().flush();
	loop {
		let mut input = String::new();
		let _ = stdin.read_line(&mut input);
		let input = input.trim().to_lowercase();
		if input == "y" || input.is_empty() {
			break;
		} else if input == "n" {
			return Err(Error::UserCancelled);
		} else {
			println!("\nInput invalid.")
		}
	}
	
	let packages = added.iter()
		.map(|m| db.get_from_unique_id(m).expect("metadb package not found"))
		.collect::<Vec<_>>();

	{
		let download_results = ckan_rs_core::installation::download::download_packages_content(config, packages.as_slice(), false).await;
		for result in &download_results {
			if result.1.is_err() { log::error!("failed to download package {} {:?}", result.0.identifier.identifier, result.1)}
		}
		if download_results.iter().any(|e| e.1.is_err()) {
			return Err(Error::Download);
		}
	}

	for package in packages {
		ckan_rs_core::installation::content::extract_content_to_deployment(config, &instance, package).unwrap();
	}

	instance.redeploy_packages(db).await.map_err(|_| Error::Deployment)?;
	instance.save_to_disk(config)?;

	Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("CKAN-rs error: {0}")]
	CKANrsError(#[from] ckan_rs_core::Error),
	#[error("Missing argument")]
	MissingArgument,
	#[error("Resolver")]
	Resolver,
	#[error("Download")]
	Download,
	#[error("Deployment failed")]
	Deployment,
	#[error("User cancelled an action")]
	UserCancelled,
}