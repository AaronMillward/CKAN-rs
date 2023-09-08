#![allow(non_snake_case)]
use dioxus::prelude::*;

#[inline_props]
pub fn ModDetails(cx: Scope, package: ckan_rs::metadb::package::Package) -> Element {
	let author_str = package.author.join(", ");
	let download_size = package.download_size.unwrap_or(0);
	cx.render(rsx! {
		div {
			"{package.name}"
			br {}
			"{author_str}"
			br {}
			"{package.blurb}"
			br {}
			"{download_size}"
		}
	})
}