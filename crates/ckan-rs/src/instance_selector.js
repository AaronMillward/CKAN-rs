const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", async () => {
	let list = document.getElementById("instance-selector");
	let instances = await invoke("get_instances");
	console.log(instances)
	for(let i in instances) {
		let instance = instances[i];

		let d = document.createElement("div");
		d.className = "instance-card card"
		// d.onclick = () => {
		// 	invoke("open_mod_detail_window", { package: packages[m] } );
		// }

		let title = document.createElement("h3");
		title.innerText = instance.name
		d.appendChild(title);

		let path = document.createElement("span");
		path.innerText = instance.path
		d.appendChild(path);

		list.appendChild(d);
	}
});