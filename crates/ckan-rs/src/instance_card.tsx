import './styles.css';

export default function InstanceCard({instance, onClickCard}: any) {
	function handle_click_card() {
		onClickCard(instance)
	}

	return (
		<div className="card instance-card" onClick={handle_click_card}>
			<h3 className="title">{instance.name}</h3>
			<span className="path">{instance.path}</span>
		</div>
	)
}