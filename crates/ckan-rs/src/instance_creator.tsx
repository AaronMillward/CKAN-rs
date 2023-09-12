import './styles.css';

import { useState } from 'react';

import { invoke } from '@tauri-apps/api'
import DirectorySelector from './directory_selector';

export default function InstanceCreator() {
	const [result, setResult] = useState("");

	function handleSubmit(e: any) {
		e.preventDefault();
		
		const form = Object.fromEntries(new FormData(e.target).entries());

		invoke("create_instance", {
			name: form.name,
			instanceRoot: form.path,
			instanceDeployment: form.deployment,
		})
		.then(() => setResult(`Created new instance ${form.name} at ${form.path}`))
		.catch(error => {
			let msg = `Failed to create new instance "${form.name}" at "${form.path}": ${error}`;
			setResult(msg)
			console.log(msg)
		});
	}

	return (
		<div className="card instance-creator">
			<h2>Instance Creator</h2>

			<form method="post" onSubmit={handleSubmit}>
				<label className="entry">
					Instance Name
					<input name="name" type="text" />
				</label>

				<label className="entry">
					Game Directory Path
					<DirectorySelector inputName="path" eventName="path-directory-selected" />
				</label>

				<label className='entry'>
					Mod Deployment Directory Path
					<DirectorySelector inputName="deployment" eventName="deployment-directory-selected" />
				</label>

				<div className="entry">
					<input type="submit" value="Create" />
					<span id="result">{result}</span>
				</div>
			</form>
		</div>
	)
}