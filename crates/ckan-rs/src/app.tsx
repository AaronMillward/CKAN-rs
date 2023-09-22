import './styles.css';

import InstanceSelector from './instance_selector';
import InstanceCreator from './instance_creator';
import PackageInstaller from './package/package_installer';
import { useState } from 'react';
import React from 'react';
import NavBar, { NavBarFiller } from './navbar';


export enum AppScreen {
	InstanceSelector,
	InstanceCreator,
	PackageInstaller,
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
		case AppScreen.PackageInstaller:
			content = <PackageInstaller />;
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