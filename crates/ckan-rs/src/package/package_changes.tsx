import { useContext } from 'react';
import '../styles.css'
import PackageCard from './package_card';
import { invoke } from '@tauri-apps/api'
import { InstanceContext } from '../app';

export default function PackageChanges({changelist}: any) {
	const { instance, setInstance } = useContext(InstanceContext);
	
	let cards = changelist.map((pack: any) => {
		return <PackageCard key={pack[0]} pack={pack[1]} showInstallControls={false}/>
	})

	function applyChanges() {
		console.log(changelist)
		invoke("change_packages", {instanceName: instance.name, add: changelist.map((p: any) => {console.log(p[1].identifier); return p[1].identifier}), remove: []})
			.then(() => {console.log("worked!")})
			.catch(error => {
				console.log("ERROR:" + error)
			});
		changelist
	}

	return (
		<div>
			<h1>Package Changes</h1>
			<div id="package-list">
				{cards}
			</div>
			<div onClick={applyChanges}>Apply</div>
		</div>
	)
}