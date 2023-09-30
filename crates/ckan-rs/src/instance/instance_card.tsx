import '../styles.css';

export default function InstanceCard({instance, onClickCard}: any) {
	return (
		<div className="card instance-card" onClick={() => {onClickCard(instance)}}>
			<h3 className="title">{instance.name}</h3>
			<span className="path">{instance.path}</span>
		</div>
	)
}