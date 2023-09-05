#![allow(non_snake_case)]
use dioxus::prelude::*;

#[inline_props]
pub fn ModCard<'a>(cx: Scope, package: &'a ckan_rs::metadb::package::Package) -> Element {
	let author = package.author.get(0).map(|s| s.as_str()).unwrap_or("No Author");

	cx.render(rsx!(
		div {
			class: "ModCard",
			div {
				margin_left: "1em",
				div {
					"{package.name}"
					br {}
					"{package.blurb}"
					br {}
					"{author}"
				}
			}
			div {
				margin_left: "auto",
				margin_right: "1em",
				"{package.identifier.version.to_string()}"
			}
		}
	))
}