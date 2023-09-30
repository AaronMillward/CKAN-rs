import './styles.css';

import InstanceSelector from './instance/instance_selector';
import InstanceCreator from './instance/instance_creator';
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
export const InstanceContext = React.createContext<any>(null);

export default function App() {
	const [appScreen, setAppScreen] = useState<any>(AppScreen.InstanceSelector)
	const [instance, setInstance] = useState<any>(null)
	
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
			<InstanceContext.Provider value={{instance: instance, setInstance: setInstance}}>
			<AppScreenContext.Provider value={{ appScreen: appScreen, setAppScreen: setAppScreen }}>
				<NavBar />
				<NavBarFiller />
				{content}
			</AppScreenContext.Provider>
			</InstanceContext.Provider>
		</div>
	)
}