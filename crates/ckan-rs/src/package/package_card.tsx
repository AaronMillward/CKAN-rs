import { useState } from 'react';
import '../styles.css';

export default function PackageCard({id, pack, showInstallControls, onInstall, onUninstall, onClickCard}: any) {
	const [selected, setSelected] = useState(false);

	function handle_click_install() {
		setSelected(!selected);
		selected ? onUninstall([id, pack]) : onInstall([id, pack]);
	}

	function handle_click_card() {
		onClickCard(pack)
	}

	let render = null;

	if(showInstallControls) {
		let which =
		 selected ? 
		<div className="card-right package-controls package-controls-uninstall" onClick={handle_click_install}/>
		:
		<div className="card-right package-controls package-controls-install" onClick={handle_click_install}/>
		;

		render =
		<div className='package-card-container'>
			<div className="card package-card card-left" onClick={handle_click_card}>
				<h2 className="title">{pack.name}</h2>
				<span className="authors">{pack.author.join(", ")}</span>
				<span className="version">{pack.identifier.version.version}</span>
			</div>
			{which}
		</div>;
	} else {
		render = 
		<div className='package-card-container'>
			<div className="card package-card card-left card-right" onClick={handle_click_card}>
				<h2 className="title">{pack.name}</h2>
				<span className="authors">{pack.author.join(", ")}</span>
				<span className="version">{pack.identifier.version.version}</span>
			</div>
		</div>
	}

	return (
		render
	)
}