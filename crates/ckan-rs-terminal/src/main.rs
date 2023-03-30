
fn create_instance(db: &ckan_rs::MetaDB, opts: getopts::Matches) {
	let instance = match opts.free.get(2) {
		Some(p) => std::path::PathBuf::from(p),
		None => { log::error!("Instance path not provided."); return },
	};

	let deployment_dir = {
		let base = instance.parent().expect("Instance should have a parent directory.");
		let instance_dir_name = instance.file_name().expect("instance path should not terminate in \"..\"");
		base.join(instance_dir_name.to_str().expect("instance directory name should be valid unicode.").to_owned() + "-deployment")
	};

	/* TODO: Check if instance exists already */

	match ckan_rs::game_instance::GameInstance::new(db.get_game_builds(), instance, deployment_dir) {
		Ok(ins) => ins,
		Err(e) => match e {
			ckan_rs::Error::IO(e) => {
				log::error!("Game instance is missing required files. possibly not a valid instance? IO Error: {}", e);
				return
			},
			ckan_rs::Error::Parse(e) => {
				log::error!("Failed to get build id for instance. IO Error: {}", e);
				return
			},
			_ => unimplemented!("unexpected error kind when creating game instance."),
		},
	};

	todo!()
}

fn main() {
	let program;
	let mut opts;
	
	/* Parse console input */ 
	let parsed_options = {
		let args: Vec<String> = std::env::args().collect();
		program = args[0].clone();
	
		opts = getopts::Options::new();
		opts.optflag( "h", "help",       "Show help");
		opts.optflag( "v", "verbose",    "Increased vebosity");
		opts.parsing_style(getopts::ParsingStyle::FloatingFrees);
	
		let parsed_options = match opts.parse(&args[1..]) {
			Ok(m)  => { m }
			Err(e) => { log::error!("Unable to parse options: {}", e); return }
		};
		
		if parsed_options.opt_present("h") {
			eprintln!("{}", opts.usage(""));
			return;
		}

		parsed_options
	};

	let config = ckan_rs::CkanRsConfig::load_from_disk().unwrap_or_else(|e| {
		log::warn!("Failed to read config file: {}", e);
		log::warn!("Using default config.");
		ckan_rs::CkanRsConfig::default()
	});

	if parsed_options.free.is_empty() {
		/* TODO: Error message */
		return;
	}

	if parsed_options.free.get(0).unwrap() == "instance" {
		if parsed_options.free.get(1).unwrap() == "create" {
			let db = match ckan_rs::MetaDB::load_from_disk(&config) {
				Ok(db) => db,
				Err(e) => {
					log::error!("Failed to open MetaDB: {}", e); return;
				}
			};
			create_instance(&db, parsed_options);
		}
	} else if parsed_options.free.get(0).unwrap() == "install" {
		
	}
	
}
