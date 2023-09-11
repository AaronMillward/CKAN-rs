const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", async () => {
	let list = document.getElementById("package-list");
	let packages = await invoke("get_compatiable_packages");
	
	let template = document.getElementById("package-card-template");
	let item = template.content.querySelector("div");

	for(let p in packages) {
		let pack = packages[p];
		let card = document.importNode(item, true);
		card.getElementsByClassName("title")[0].innerText = pack.name
		card.getElementsByClassName("authors")[0].innerText = pack.author.join(", ")
		card.getElementsByClassName("version")[0].innerText = pack.identifier.version.version

		card.onclick = () => {
			invoke("open_package_detail_window", { package: pack } );
		}

		list.appendChild(card);
	}
});