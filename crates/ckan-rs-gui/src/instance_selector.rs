#![allow(non_snake_case)]
use dioxus::prelude::*;

#[inline_props]
pub fn InstanceSelector<'a>(cx: Scope, on_click: EventHandler<'a, MouseEvent>,) -> Element<'a> {
	cx.render(rsx!(
		div {
			onclick: move |evt| cx.props.on_click.call(evt),
			"InstanceSelector"
		}
	))
}