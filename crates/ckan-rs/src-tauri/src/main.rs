// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::State;

mod package_list;

#[tauri::command]
fn get_instances(config: State<ckan_rs_core::Config>) -> Vec<ckan_rs_core::game_instance::GameInstance> {
	let dir = match std::fs::read_dir(config.data_dir().join("instances")) {
		Ok(dir) => dir,
		Err(_) => return Vec::<_>::new(),
	}; 
	dir
		.filter_map(|f| f.ok())
		.map(|f| ckan_rs_core::game_instance::GameInstance::load_by_file(f.path()))
		.filter_map(|f| f.ok())
		.collect::<Vec<_>>()
}

fn main() {
	let config = ckan_rs_core::Config::load_from_disk().unwrap_or_default();
	let metadb = ckan_rs_core::MetaDB::load_from_disk(&config).unwrap();

	tauri::Builder::default()
		.manage(config)
		.manage(metadb)
		.invoke_handler(tauri::generate_handler![
			get_instances,
			package_list::get_compatiable_packages,
			package_list::open_mod_detail_window,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
