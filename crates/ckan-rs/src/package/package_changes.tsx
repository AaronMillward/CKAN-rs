import '../styles.css'
import PackageCard from './package_card';

export default function PackageChanges({changelist}: any) {
	let cards = changelist.map((pack: any) => {
		return <PackageCard key={pack[0]} pack={pack[1]} showInstallControls={false}/>
	})

	return (
		<div>
			<h1>Package Changes</h1>
			<div id="package-list">
				{cards}
			</div>
		</div>
	)
}