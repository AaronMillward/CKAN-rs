const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

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