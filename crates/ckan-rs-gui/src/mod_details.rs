#![allow(non_snake_case)]
use dioxus::prelude::*;

#[inline_props]
pub fn ModDetails(cx: Scope, package: ckan_rs::metadb::package::Package) -> Element {
	let author_str = package.author.join(", ");
	cx.render(rsx! {
		div {
			"{package.name}"
			"{author_str}"
			"{package.blurb}"
			if let Some(download_size) = package.download_size {
				"{download_size}"
			}
		}
	})
}