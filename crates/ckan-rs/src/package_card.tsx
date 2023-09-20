import { useState } from 'react';
import './styles.css';

export default function PackageCard({pack, onInstall, onUninstall, onClickCard}: any) {
	const [selected, setSelected] = useState(false);

	function handle_click_install() {
		setSelected(!selected);
		selected ? onUninstall(pack) : onInstall(pack);
	}

	function handle_click_card() {
		onClickCard(pack)
	}

	let controls = selected ? 
		<div className="card-right package-controls package-controls-uninstall" onClick={handle_click_install}/>
		:
		<div className="card-right package-controls package-controls-install" onClick={handle_click_install}/>
		;

	return (
		<div className='package-card-container'>
			<div className="card package-card card-left" onClick={handle_click_card}>
				<h2 className="title">{pack.name}</h2>
				<span className="authors">{pack.author.join(", ")}</span>
				<span className="version">{pack.identifier.version.version}</span>
			</div>
			{controls}
		</div>
	)
}