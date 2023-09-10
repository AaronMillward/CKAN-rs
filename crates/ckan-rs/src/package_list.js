const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", async () => {
	let modlist = document.getElementById("modlist");
	let packages = await invoke("get_compatiable_packages");
	console.log(packages[0])
	// console.log(packages)
	for(let m in packages) {
		let d = document.createElement("div");
		d.className = "mod-card"
		d.onclick = () => {
			invoke("open_mod_detail_window", { package: packages[m] } );
		}

		let title = document.createElement("h2");
		title.innerText = packages[m].name
		d.appendChild(title);

		let author = document.createElement("span");
		author.innerText = packages[m].author.join(", ")
		d.appendChild(author);

		let version = document.createElement("span");
		version.className = "mod-card-version-number"
		version.innerText = packages[m].identifier.version.version
		d.appendChild(version);

		modlist.appendChild(d);
	}
});