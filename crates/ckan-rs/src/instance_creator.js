const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

async function select_game_dir() {
	await invoke("select_directory", {
		event: "instance-directory-selected"
	});
}

async function select_deployment_dir() {
	await invoke("select_directory", {
		event: "deployment-directory-selected"
	});
}

async function create_instance() {
	let name = document.getElementById("name").value;
	let instance_root = document.getElementById("path").value;
	let instance_deployment = document.getElementById("deployment").value;
	let res = invoke("create_instance", {
		name: name,
		instanceRoot: instance_root,
		instanceDeployment: instance_deployment,
	});
	res.then(
		() => {
			document.getElementById("result").innerText = "Created new instance name at instance_root";
		},
		(reason) => {
			document.getElementById("result").innerText = `Failed to create new instance "${name}" at "${instance_root}": ${reason}`;
		}
	)
}

const unlisten = await listen('instance-directory-selected', (event) => {
	document.getElementById("path").value = event.payload;
})

const unlisten2 = await listen('deployment-directory-selected', (event) => {
	document.getElementById("deployment").value = event.payload;
})

document.getElementById("path-picker").addEventListener("click", select_game_dir, false)
document.getElementById("deployment-picker").addEventListener("click", select_deployment_dir, false)

document.getElementById("submit").addEventListener("click", create_instance, false)