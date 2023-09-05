#![allow(non_snake_case)]
use dioxus::prelude::*;

use ckan_rs::metadb::package::KspVersionReal;

use super::mod_card::ModCard;

#[inline_props]
pub fn ModList<'a>(cx: Scope, db: &'a UseFuture<Result<ckan_rs::MetaDB, ckan_rs::Error>>) -> Element<'a> {
	match db.value() {
		Some(Ok(db)) => {
			let packages: Vec<_> = db.get_packages().iter()
				.filter(|p| p.ksp_version.is_version_compatible(&KspVersionReal::try_from("1.12.3").unwrap(), false))
				.collect();
			render! {
				div {
					class: "ModList",
					for package in packages {
						ModCard { package: package }
					}
				}
			}
		},
		Some(Err(e)) => {
			cx.render(rsx!(
				div {
					"CKAN-RS"
				}
				div {
					"Failed to open MetaDB due to error {e}"
				}
			))
		},
		None => {
			render! {
				div {
					"CKAN-RS"
				}
				div {
					width: "100%",
					height: "100%",
					background_color: "red",
					"Loading MetaDB"
				}
			}
		},
	}
}