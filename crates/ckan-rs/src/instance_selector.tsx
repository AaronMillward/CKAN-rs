import './styles.css';
import { useContext, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api'
import InstanceCard from './instance_card';
import { AppScreen, AppScreenContext } from './app';

export default function InstanceSelector() {
	const { appScreen, setAppScreen } = useContext(AppScreenContext);
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

	let list = <div className="instance-list">{cards}</div>;

	let no_list = <div> No instances found. </div>;

	let content = cards.length == 0 ? no_list : list;

	return (
		<div>
			<h1>Instance Selector</h1>
			<div className='instance-selector'>
				<div className='instance-content'>
					{content}
				</div>
				<input type="button" value="Create Instance" onClick={() => {setAppScreen(AppScreen.InstanceCreator)}}/>
			</div>
		</div>
	)
}