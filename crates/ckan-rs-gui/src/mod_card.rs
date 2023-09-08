#![allow(non_snake_case)]

use dioxus::prelude::*;

use ckan_rs::metadb::package::Package;

pub struct ModCardSelectedEvent<'a> {
	pub package: &'a Package,
}

#[derive(Props)]
pub struct ModCardProps<'a> {
	package: &'a Package,
	on_selected: Option<EventHandler<'a, ModCardSelectedEvent<'a>>>,
}

pub fn ModCard<'a>(cx: Scope<'a, ModCardProps>) -> Element<'a> {
	let package = cx.props.package;
	let author = package.author.get(0).map(|s| s.as_str()).unwrap_or_else(|| "No Author");
	
	cx.render(rsx!(
		div {
			class: "ModCard",
			onclick: move |_| {
				if let Some(on_selected) = &cx.props.on_selected {
					on_selected.call(ModCardSelectedEvent { package })
				}
			},
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