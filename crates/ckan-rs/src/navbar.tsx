import './styles.css';

import { AppScreen, AppScreenContext } from './app';
import { useContext, useEffect } from 'react';


export default function NavBar() {
	const { appScreen, setAppScreen } = useContext(AppScreenContext);

	return (
		<div className='navigation fixed'>
			<h1>CKAN-RS</h1>
			<nav>
				<button className='nav-button' onClick={() => {setAppScreen(AppScreen.InstanceSelector)}}>Instances</button>
				<button className='nav-button' onClick={() => {setAppScreen(AppScreen.PackageInstaller)}}>Packages</button>
			</nav>
		</div>
	)
}

/// Component that fills the space left by NavBar so that the page content isn't occluded.
export function NavBarFiller() {
	return ( <div className='navigation filler' /> )
}