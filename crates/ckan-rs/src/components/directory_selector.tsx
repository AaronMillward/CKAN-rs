import '../styles.css';

import { useEffect, useState } from 'react';

import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'

export default function DirectorySelector({inputName, eventName}: any) {
	const [val, setVal] = useState("");

	useEffect(() => {
		const unlisten = listen<string>(eventName, (event) => {
			console.log('Received event:', event.payload);
			setVal(event.payload);
		});
	
		return () => {
			unlisten.then(f => f());
		};
	}, []);

	function select_directory() {
		invoke("select_directory", {
			event: eventName
		})
	}

	return (
		<div className="system-file-chooser">
			<input name={inputName} type="text" value={val} onChange={e => setVal(e.target.value)} />
			<input className="system" type="button" value="Select from system" onClick={select_directory}/>
		</div>
	)
}