fn print_usage(program: &str, opts: getopts::Options) {
	println!("{}", opts.usage(&format!("Usage: {} [options] INSTANCE", program)));
}

fn main() {
	/* Parse console input */ 
	let (instance, parsed_options) = {
		let args: Vec<String> = std::env::args().collect();
		let program = args[0].clone();
	
		let mut opts = getopts::Options::new();
		opts.optflag( "h", "help",       "show help");
		opts.optflag( "v", "verbose",    "increased vebosity");
		opts.optmulti("s", "install-package", "Adds a package and its dependencies to the instance.", "PACKAGE");
		opts.optmulti("r", "remove-package", "Remove a package from the instance.", "PACKAGE");
		opts.parsing_style(getopts::ParsingStyle::FloatingFrees);
	
		let parsed_options = match opts.parse(&args[1..]) {
			Ok(m)  => { m }
			Err(f) => { log::error!("Unable to parse options. {}", f); print_usage(&program, opts); return }
		};
		
		if parsed_options.opt_present("h") {
			print_usage(&program, opts);
			return;
		}
	
		let instance = match parsed_options.free.get(0) {
			Some(p) => std::path::PathBuf::from(p),
			None => { log::error!("Instance path not provided."); print_usage(&program, opts); return },
		};
	
		let deployment_dir = {
			let base = instance.parent().expect("Instance should have a parent directory.");
			let instance_dir_name = instance.file_name().expect("instance path should not terminate in \"..\"");
			base.join(instance_dir_name.to_str().expect("instance directory name should be valid unicode.").to_owned() + "-deployment")
		};
	
		let mut instance = match ckan_rs::game_instance::GameInstance::new(instance, deployment_dir) {
			Ok(ins) => ins,
			Err(e) => match e {
				ckan_rs::game_instance::GameInstanceError::RequiredFilesMissing(_) => {
					log::error!("Game instance is missing required files. possibly not an instance?");
					return
				},
			},
		};

		(instance, parsed_options)
	};

	let install = parsed_options.opt_strs("s");
	let remove = parsed_options.opt_strs("r");

}
