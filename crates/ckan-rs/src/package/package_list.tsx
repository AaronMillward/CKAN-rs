import '../styles.css';
import { useContext, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api'
import PackageCard from './package_card';
import { InstanceContext } from '../app';

export default function PackageList({OnPackagesChanged}: any) {
	const [packages, setPackages] = useState(Array<any>);
	const [installedPackages, setInstalledPackages] = useState(Array<any>);
	const { instance, setInstance } = useContext(InstanceContext);

	useEffect(() => {
		invoke("get_compatiable_packages")
			.then((packages: any) => {setPackages(packages)})
			.catch(error => {
				console.log(error)
			});
	}, []);

	useEffect(() => {
		invoke("get_installed_packages", {instance_name: instance.name})
			.then((packages: any) => {setInstalledPackages(packages)})
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
		let is_package_installed = pack.identifier.identifier in installedPackages.map((p) => p.identifier.identifier);
		return <PackageCard key={i} id={i} pack={pack} showInstallControls={true} isInstalledAlready={is_package_installed} onClickCard={handle_card_clicked} onInstall={handle_installed} onUninstall={handle_uninstalled}/>
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