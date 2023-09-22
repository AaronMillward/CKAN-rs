import React from "react";
import ReactDOM from "react-dom/client";
import "../styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<React.StrictMode>
		<PackageDetail />
	</React.StrictMode>,
);

import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event'

export default function PackageDetail() {
	const [val, setVal] = useState({});

	useEffect(() => {
		const unlisten = listen<any>('show-mod-detail', (event) => {
			setVal(event.payload);
		})

		return () => {
			unlisten.then(f => f());
		};
	}, []);

	let td = Object.entries(val).map(([k,v]: [string, any]) => {
		let data = v == null ? "" : v.toString();
		return <tr key={k}><td>{k}</td><td>{data}</td></tr>
	});

	console.log(td)

	return (
		<table>
			<thead>
				<tr><td><b>Property</b></td><td><b>Value</b></td></tr>
			</thead>
			<tbody>{td}</tbody>
		</table>
	)
}