import './styles.css';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api'
import PackageCard from './package_card';

export default function PackageList() {
	const [packages, setPackages] = useState(Array<any>);

	useEffect(() => {
		invoke("get_compatiable_packages")
			.then((packages: any) => setPackages(packages))
			.catch(error => {
				console.log(error)
			});
	}, []);

	function handle_card_clicked(pack: any) {
		invoke("open_package_detail_window", { package: pack } );
	}

	function handle_installed(pack: any) { console.log(`installing ${pack.identifier.identifier}`); }
	function handle_uninstalled(pack: any) { console.log(`uninstalling ${pack.identifier.identifier}`); }

	let cards = packages.map((pack) => {
		return <PackageCard pack={pack} onClickCard={handle_card_clicked} onInstall={handle_installed} onUninstall={handle_uninstalled}/>
	})

	return (
		<div>
			<h1>Package List</h1>
			<div id="package-list">
				{cards}
			</div>
		</div>
	)
}