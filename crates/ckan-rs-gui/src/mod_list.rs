#![allow(non_snake_case)]
use dioxus::prelude::*;

use ckan_rs::metadb::package::KspVersionReal;

use super::mod_card::*;

#[derive(Props)]
pub struct ModListProps<'a> {
	db: &'a UseFuture<Result<ckan_rs::MetaDB, ckan_rs::Error>>,
}

pub fn ModList<'a>(cx: Scope<'a, ModListProps>) -> Element<'a> {
	match cx.props.db.value() {
		Some(Ok(db)) => {
			let packages = db.get_packages().iter()
				.filter(|p| p.ksp_version.is_version_compatible(&KspVersionReal::try_from("1.12.3").unwrap(), false));

			let window = dioxus_desktop::use_window(cx);

			let cards_rendered = packages.map(|package| {
				rsx!(
					ModCard {
						package: package,
						on_selected: move |evt: ModCardSelectedEvent| {
							/* NOTE: Although I hate using clone here it's much cleaner than changing the whole library to allow shared ownership of packages. */
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