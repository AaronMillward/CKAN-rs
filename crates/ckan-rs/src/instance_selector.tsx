import './styles.css';

import { useEffect, useState } from 'react';

import { invoke } from '@tauri-apps/api'

import InstanceCard from './instance_card';

function InstanceSelector() {
	const [instances, setInstances] = useState(Array<any>);

	useEffect(() => {
		invoke("get_instances")
			.then((userList: any) => setInstances(userList))
			.catch(error => {
				console.log(error)
			});
	}, []);


	let cards = instances.map((instance) => {
		return <InstanceCard instance={instance} />
	})

	return (
		<div>
			<h1>Instance Selector</h1>
			<div id="instance-selector">
				{cards}
			</div>
		</div>
	)
}

export default InstanceSelector