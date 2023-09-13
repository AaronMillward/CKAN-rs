import './styles.css';

import InstanceSelector from './instance_selector';
import PackageList from './package_list';
import InstanceCreator from './instance_creator';
import { useState } from 'react';
import React from 'react';
import NavBar, { NavBarFiller } from './navbar';

export enum AppScreen {
	InstanceSelector,
	InstanceCreator,
	PackageList,
}

export const AppScreenContext = React.createContext<any>(null);

export default function App() {
	const [appScreen, setAppScreen] = useState<any>(AppScreen.InstanceSelector)
	
	let content = <h1>UNINITIALIZED</h1>;
	switch(appScreen) {
		case AppScreen.InstanceSelector:
			content = <InstanceSelector />;
			break;
		case AppScreen.InstanceCreator:
			content = <InstanceCreator />;
			break;
		case AppScreen.PackageList:
			content = <PackageList />;
			break;
	}

	return (
		<div>
			<AppScreenContext.Provider value={{ appScreen: appScreen, setAppScreen: setAppScreen }}>
				<NavBar />
				<NavBarFiller />
				{content}
			</AppScreenContext.Provider>
		</div>
	)
}