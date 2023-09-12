import './styles.css';

import InstanceSelector from './instance_selector';
import PackageList from './package_list';
import InstanceCreator from './instance_creator';

enum AppMenu {
	InstanceSelector,
	InstanceCreator,
	PackageList,
}

let state = AppMenu.PackageList;

function App() {
	let content = <h1>UNINITIALIZED</h1>;
	switch(state) {
		case AppMenu.InstanceSelector:
			content = <InstanceSelector />;
			break;
		case AppMenu.InstanceCreator:
			content = <InstanceCreator />;
			break;
		case AppMenu.PackageList:
			content = <PackageList />;
			break;
	}

	return (
		<div>
			<h1>CKAN-RS</h1>
			{content}
		</div>
	)
}

export default App