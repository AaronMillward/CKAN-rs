import '../styles.css';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api'
import PackageCard from './package_card';

export default function PackageList({OnPackagesChanged}: any) {
	const [packages, setPackages] = useState(Array<any>);

	useEffect(() => {
		invoke("get_compatiable_packages")
			.then((packages: any) => {setPackages(packages)})
			.catch(error => {
				console.log(error)
			});
	}, []);

	function handle_card_clicked(pack: any) {
		invoke("open_package_detail_window", { package: pack } );
	}

	function handle_installed(pack: any) { OnPackagesChanged(pack,null) }
	function handle_uninstalled(pack: any) { OnPackagesChanged(null,pack) }

	let cards = packages.map((pack, i) => {
		return <PackageCard key={i} id={i} pack={pack} showInstallControls={true} onClickCard={handle_card_clicked} onInstall={handle_installed} onUninstall={handle_uninstalled}/>
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