#![allow(non_snake_case)]
use dioxus::prelude::*;

fn create_instance(game_directory: &str) {
	println!("Create instance at {}", game_directory)
}

pub fn InstanceCreator(cx: Scope) -> Element {
	cx.render(rsx! {
		input {
			r#type: "file",
			accept: "",
			multiple: false,
			directory: true,
			onchange: |evt| {
				async move {
					if let Some(file_engine) = &evt.files {
						let files = file_engine.files();
						if let Some(dir) = files.first() {
							create_instance(dir)
						}
					}
				}
			},
		},
	})
}