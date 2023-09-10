const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
console.log(listen)
// // listen to the `click` event and get a function to remove the event listener
// // there's also a `once` function that subscribes to an event and automatically unsubscribes the listener on the first event
const unlisten = await listen('show-mod-detail', (event) => {
	let tbody = document.getElementById("details");

	for(let prop_name in event.payload) {
		let tr = document.createElement("tr");
		let prop = document.createElement("td");
		prop.innerText = prop_name;
		let val = document.createElement("td");
		val.innerText = event.payload[prop_name];
		tr.appendChild(prop)
		tr.appendChild(val)
		tbody.appendChild(tr)
	}
})

// window.addEventListener("DOMContentLoaded", async () => {
// 	let modlist = document.getElementById("modlist");
// 	let packages = await invoke("get_compatiable_packages");
// 	console.log(packages[0])
// 	// console.log(packages)
// 	for(let m in packages) {
// 		let d = document.createElement("div");
// 		d.className = "mod-card"
// 		d.onclick = () => {
// 			invoke("open_mod_detail_window", { package: packages[m] } );
// 		}

// 		let title = document.createElement("h2");
// 		title.innerText = packages[m].name
// 		d.appendChild(title);

// 		let author = document.createElement("span");
// 		author.innerText = packages[m].author.join(", ")
// 		d.appendChild(author);

// 		let version = document.createElement("span");
// 		version.className = "mod-card-version-number"
// 		version.innerText = packages[m].identifier.version.version
// 		d.appendChild(version);

// 		modlist.appendChild(d);
// 	}
// });