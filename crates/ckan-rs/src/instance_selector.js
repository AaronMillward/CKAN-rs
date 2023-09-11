const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", async () => {
	let list = document.getElementById("instance-selector");
	let instances = await invoke("get_instances");

	let template = document.getElementById("instance-card-template");
	let item = template.content.querySelector("div");

	for(let i in instances) {
		let instance = instances[i];
		let card = document.importNode(item, true);
		card.getElementsByClassName("title")[0].innerText = instance.name;
		card.getElementsByClassName("path")[0].innerText = instance.path

		// card.onclick = () => {
		// 	invoke("open_package_detail_window", { package: packages[m] } );
		// }

		list.appendChild(card);
	}
});