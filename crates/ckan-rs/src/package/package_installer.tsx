import '../styles.css';
import {useState} from 'react';
import PackageChanges from './package_changes';
import PackageList from './package_list';

export default function PackageInstaller() {
	const [changelist, setChangelist] = useState(Array<any>);

	function handlePackagesChanged(install: any, uninstall: any) {
		let n = [...changelist];
		if(install != null) { n.push(install) }
		if(uninstall != null) { n = n.filter(p => p[1].identifier.identifier !== uninstall[1].identifier.identifier) }
		setChangelist(n)
	}

	return (
		<div style={{display: "flex"}}>
			<PackageList OnPackagesChanged={handlePackagesChanged}/>
			<PackageChanges changelist={changelist} />
		</div>
	)
}