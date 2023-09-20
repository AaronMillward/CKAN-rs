import { useState } from 'react';
import './styles.css';
import { invoke } from '@tauri-apps/api'

export default function PackageCard({pack}: any) {
	const [selected, setSelected] = useState(false);

	function click_handler_info() {
		invoke("open_package_detail_window", { package: pack } );
	}

	function click_handler_install() {
		setSelected(!selected)
	}

	let controls = selected ? 
		<div className="card-right package-controls package-controls-uninstall" onClick={click_handler_install}/>
		:
		<div className="card-right package-controls package-controls-install" onClick={click_handler_install}/>
		;

	return (
		<div className='package-card-container'>
			<div className="card package-card card-left" onClick={click_handler_info}>
				<h2 className="title">{pack.name}</h2>
				<span className="authors">{pack.author.join(", ")}</span>
				<span className="version">{pack.identifier.version.version}</span>
			</div>
			{controls}
		</div>
	)
}