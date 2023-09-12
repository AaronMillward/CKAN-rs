import './styles.css';
import { invoke } from '@tauri-apps/api'

export default function PackageCard({pack}: any) {
	function click_handler() {
		invoke("open_package_detail_window", { package: pack } );
	}

	return (
		<div className="card package-card" onClick={click_handler}>
			<h2 className="title">{pack.name}</h2>
			<span className="authors">{pack.author.join(", ")}</span>
			<span className="version">{pack.identifier.version.version}</span>
		</div>
	)
}