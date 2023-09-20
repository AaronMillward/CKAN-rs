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

	let cards = packages.map((pack) => {
		return <PackageCard pack={pack} />
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