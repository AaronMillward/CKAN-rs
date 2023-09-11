use tauri::State;

use ckan_rs_core::metadb::package::KspVersionReal;

/// The regular [`ckan_rs_core::game_instance::GameInstance`] struct has properties which can't be serialized to JSON by Tauri. which leads to a message "key must be a string"
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GameInstanceInfo {
	name: String,
	path: std::path::PathBuf,
	compatible_ksp_versions: Vec<KspVersionReal>,
	deployment_dir: std::path::PathBuf,
}

#[tauri::command]
pub fn get_instances(config: State<ckan_rs_core::Config>) -> Vec<GameInstanceInfo> {
	let dir = match std::fs::read_dir(config.data_dir().join("instances")) {
		Ok(dir) => dir,
		Err(_) => return Vec::<_>::new(),
	}; 
	dir
		.filter_map(|f| f.ok())
		.map(|f| ckan_rs_core::game_instance::GameInstance::load_by_file(f.path()))
		.filter_map(|f| f.ok())
		.map(|i| GameInstanceInfo {
			name: i.name().to_string(),
			path: i.path().to_path_buf(),
			deployment_dir: i.deployment_dir.clone(),
			compatible_ksp_versions: i.compatible_ksp_versions().clone(),
		})
		.collect::<_>()
}