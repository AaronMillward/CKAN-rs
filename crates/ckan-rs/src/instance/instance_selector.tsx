import '../styles.css';
import { useContext, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api'
import InstanceCard from './instance_card';
import { AppScreen, AppScreenContext } from '../app';
import { InstanceContext } from '../app';

export default function InstanceSelector() {
	const { appScreen, setAppScreen } = useContext(AppScreenContext);
	const { instance, setInstance }   = useContext(InstanceContext);
	
	const [ instances, setInstances ] = useState(Array<any>);

	useEffect(() => {
		invoke("get_instances")
			.then((userList: any) => setInstances(userList))
			.catch(error => {
				console.log(error)
			});
	}, []);

	let content;
	if(instances.length == 0){
		content = <div> No instances found. </div>;
	} else {
		function handle_click_card(instance: any) {
			setInstance(instance)
		}

		let cards = instances.map((instance) => {
			return <InstanceCard instance={instance} onClickCard={handle_click_card} />
		});
		content = <div className="instance-list">{cards}</div>;
	};

	let selected;
	if(instance == null) {
		selected = <div>No Instance Selected.</div>;
	} else {
		selected = <div>Selected: {instance.name}</div>;
	}

	return (
		<div>
			<h1>Instance Selector</h1>
			<div className='instance-selector'>
				<div className='instance-content'>
					{content}
				</div>
				{selected}
				<input type="button" value="Create Instance" onClick={() => {setAppScreen(AppScreen.InstanceCreator)}}/>
			</div>
		</div>
	)
}