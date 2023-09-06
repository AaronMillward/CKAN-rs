#![allow(non_snake_case)]
use std::rc::Rc;

use dioxus::prelude::*;

mod mod_card;
mod mod_list;

mod instance_selector;

enum MainMode {
	InstanceSelector,
	ModList,
}

fn main() {
	let config = Rc::new(ckan_rs::CkanRsConfig::load_from_disk().unwrap_or_else(|e| {
		log::warn!("Failed to read config file: {}", e);
		log::warn!("Using default config.");
		ckan_rs::CkanRsConfig::default()
	}));

	dioxus_desktop::launch_with_props(
		App,
		AppProps { ckan_config: config },
		dioxus_desktop::Config::default()
			.with_window(dioxus_desktop::WindowBuilder::default()
				.with_title("CKAN-rs")
			)
	);
}

#[inline_props]
pub fn App(cx: Scope, ckan_config: Rc<ckan_rs::CkanRsConfig>) -> Element {
	async fn genreate_and_save_new_metadb(config: &ckan_rs::CkanRsConfig) -> ckan_rs::Result<ckan_rs::MetaDB> {
		let db = ckan_rs::metadb::generate_latest().await?;
		db.save_to_disk(config)?;
		Ok(db)
	}

	async fn get_metadb_from_disk_or_generate(config: Rc<ckan_rs::CkanRsConfig>) -> ckan_rs::Result<ckan_rs::MetaDB> {
		match ckan_rs::MetaDB::load_from_disk(&config) {
			Ok(db) => Ok(db),
			Err(e) => {
				match e {
					ckan_rs::Error::IO(e) => {
						match e.kind() {
							std::io::ErrorKind::NotFound => {
								let res = genreate_and_save_new_metadb(&config).await;
								match res {
									Ok(db) => Ok(db),
									Err(e) => {
										log::error!("Failed to generate and save metadb: {}", e);
										Err(ckan_rs::Error::Validation("".into()))
									}
								}
							}
							_ => {
								log::error!("Failed to open MetaDB due to IO error: {}", e);
								Err(ckan_rs::Error::Validation("".into()))
							}
						}
					},
					ckan_rs::Error::Parse(_) => {
						log::warn!("Failed to open MetaDB due to parsing error, DB format likely changed. regenerating...");
						let res = genreate_and_save_new_metadb(&config).await;
						match res {
							Ok(db) => Ok(db), 
							Err(_) => {
								log::error!("Failed to generate metadb");
								Err(ckan_rs::Error::Validation("".into()))
							}
						}
					},
					_ => unimplemented!("unexpected error type."),
				}
			}
		}
	}

	let db = use_future(cx, (), |_| {  get_metadb_from_disk_or_generate(ckan_config.clone()) });
	// Loads forever
	// let db = use_future(cx, (), |_| { std::future::pending::<ckan_rs::Result<ckan_rs::MetaDB>>() });

	let instances = use_state(cx, || {
		let dir = match std::fs::read_dir(ckan_config.data_dir().join("instances")) {
			Ok(dir) => dir,
			Err(_) => return Vec::<_>::new(),
		}; 
		dir
			.filter_map(|f| f.ok())
			.map(|f| ckan_rs::game_instance::GameInstance::load_by_file(f.path()))
			.filter_map(|f| f.ok())
			.collect::<Vec<_>>()
	});
	
	let main_mode = use_state(cx, || MainMode::InstanceSelector);
	match main_mode.get() {
		MainMode::InstanceSelector => {
			render!(
				style { include_str!("./style.css") }
				instance_selector::InstanceSelector {
					instances: instances,
					on_instance_selected: |e: instance_selector::SelectedInstanceEvent| {
						println!("instance selected {}", e.instance);
						main_mode.set(MainMode::ModList);
					}
				}
			)
		}
		MainMode::ModList => {
			render!(
				style { include_str!("./style.css") }
				mod_list::ModList {
					db: db
				}
			)
		}
	}	
}