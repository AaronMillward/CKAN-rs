use tauri::{Window, State};


#[tauri::command]
pub fn select_directory(window: Window, event: String) {
	use tauri::api::dialog::FileDialogBuilder;
	FileDialogBuilder::new().pick_folder(move |folder_path| {
		window.emit(&event, folder_path).expect("failed to emit signal.")
	})
}

#[tauri::command]
pub fn create_instance(config: State<ckan_rs_core::Config>, metadb: State<ckan_rs_core::MetaDB>, name: String, instance_root: String, instance_deployment: String) -> Result<(), String> {
	use ckan_rs_core::game_instance::GameInstance;
	let instance_root = std::path::PathBuf::try_from(instance_root).map_err(|e| e.to_string())?;
	let instance_deployment = std::path::PathBuf::try_from(instance_deployment).map_err(|e| e.to_string())?;
	let instance = GameInstance::new(&config, metadb.get_game_builds(), name, instance_root, instance_deployment).map_err(|e| e.to_string())?;
	instance.save_to_disk(&config).map_err(|e| e.to_string())?;
	Ok(())
}