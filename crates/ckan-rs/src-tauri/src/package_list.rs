use tauri::State;
use ckan_rs_core::metadb::package::Package;
use ckan_rs_core::MetaDB;

#[tauri::command]
pub fn get_compatiable_packages(metadb: State<MetaDB>) -> Vec<Package> {
	use ckan_rs_core::metadb::package::KspVersionReal;
	metadb.get_packages()
		.iter()
		.filter(|p| p.ksp_version.is_version_compatible(&KspVersionReal::try_from("1.12.3").unwrap(), false))
		.cloned()
		.collect::<Vec<_>>()
}

#[tauri::command]
pub async fn open_mod_detail_window(handle: tauri::AppHandle, package: Package) {
	let w = tauri::WindowBuilder::new(
		&handle,
		"mod_detail",
		tauri::WindowUrl::App("mod_detail.html".parse().unwrap())
	).build().expect("failed to create mod detail window.");
	/* TODO: Replace this with on load */
	std::thread::sleep(std::time::Duration::from_millis(500));
	w.emit("show-mod-detail", package).expect("failed to show mod details.");
}