#![allow(non_snake_case)]
use dioxus::prelude::*;

use ckan_rs::metadb::package::KspVersionReal;

use super::mod_card::*;

#[inline_props]
pub fn ModList<'a>(cx: Scope, db: &'a UseFuture<Result<ckan_rs::MetaDB, ckan_rs::Error>>) -> Element<'a> {
	match db.value() {
		Some(Ok(db)) => {
			let packages: Vec<_> = db.get_packages().iter()
				.filter(|p| p.ksp_version.is_version_compatible(&KspVersionReal::try_from("1.12.3").unwrap(), false))
				.collect();

			let window = dioxus_desktop::use_window(cx);

			let cards_rendered = packages.iter().map(|package| {
				rsx!(
					ModCard {
						package: package,
						on_selected: move |evt: ModCardSelectedEvent| {
							let dom = VirtualDom::new_with_props(super::mod_details::ModDetails, super::mod_details::ModDetailsProps { package: evt.package.clone() });
							window.new_window(dom, Default::default());
						},
					}
				)
			});

			cx.render(rsx!(
				div {
					class: "ModList",
					cards_rendered
				}
			))
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