#![allow(non_snake_case)]
use dioxus::prelude::*;

use ckan_rs::game_instance::GameInstance;

pub struct SelectedInstanceEvent {
	pub instance: usize,
}

#[derive(Props)]
pub struct InstanceSelectorProps<'a> {
	instances: &'a Vec<GameInstance>,
	on_instance_selected: Option<EventHandler<'a, SelectedInstanceEvent>>
}

pub fn InstanceSelector<'a>(cx: Scope<'a, InstanceSelectorProps<'a>>) -> Element<'a> {
	let components = cx.props.instances.iter().enumerate().map(|(i, ins)| {
		let gd = ins.game_dir().to_string_lossy();
		rsx!(
			div {
				class: "InstanceListItem",
				onclick: move |_| {
					if let Some(on_instance_selected) = &cx.props.on_instance_selected {
						on_instance_selected.call(SelectedInstanceEvent { instance: i })
					}
				},
				"{gd}",
			}
		)
	});

	cx.render(rsx!(
		h1 {
			class: "MainTitle",
			"InstanceSelector",
		}
		div {
			class: "InstanceSelector",
			div {
				class: "InstanceList",
				components
			}
			div {
				class: "InstanceButtons",
				button {
					"Create Instance"
				}
				button {
					"Select Instance"
				}
			}
		}
	))
}