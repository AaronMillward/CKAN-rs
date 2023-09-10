// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod package_list;
mod instance_list;

fn main() {
	let config = ckan_rs_core::Config::load_from_disk().unwrap_or_default();
	let metadb = ckan_rs_core::MetaDB::load_from_disk(&config).unwrap();

	tauri::Builder::default()
		.manage(config)
		.manage(metadb)
		.invoke_handler(tauri::generate_handler![
			instance_list::get_instances,
			package_list::get_compatiable_packages,
			package_list::open_mod_detail_window,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
