use tauri::State;
use ckan_rs_core::metadb::package::Package;
use ckan_rs_core::metadb::package::PackageIdentifier;
use ckan_rs_core::game_instance::GameInstance;
use ckan_rs_core::MetaDB;

#[tauri::command]
pub fn get_installed_packages(config: State<ckan_rs_core::Config>, instance_name: String) -> Result<Vec<PackageIdentifier>, String> {
	let ins = GameInstance::load_by_name(&config, instance_name);
	ins.map(|i| i.enabled_packages()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_compatiable_packages(metadb: State<MetaDB>) -> Vec<Package> {
	use ckan_rs_core::metadb::package::KspVersionReal;
	let mut packages = metadb.get_packages()
		.iter()
		.filter(|p| p.ksp_version.is_version_compatible(&KspVersionReal::try_from("1.12.3").unwrap(), false))
		.collect::<Vec<_>>();
	packages.sort();
	let mut only_latest_versions = Vec::<Package>::new();
	for i in 0..packages.len() {
		let p1 = packages[i];
		// only_latest_versions.push(p1.clone());
		if let Some(p2) = packages.get(i+1) {
			if p1.identifier.identifier == p2.identifier.identifier {
				continue;
			} else {
				only_latest_versions.push(p1.clone());
			}
		} else {
			continue;
		}
	}
	only_latest_versions
}

#[tauri::command]
pub async fn open_package_detail_window(handle: tauri::AppHandle, package: Package) {
	let w = tauri::WindowBuilder::new(
		&handle,
		"mod_detail",
		tauri::WindowUrl::App("package_detail.html".parse().unwrap())
	).build().expect("failed to create mod detail window.");
	/* TODO: Replace this with on load */
	std::thread::sleep(std::time::Duration::from_millis(500));
	w.emit("show-mod-detail", package).expect("failed to show mod details.");
}