import './styles.css';

export default function InstanceCard({instance}: any) {
	return (
		<div className="card instance-card">
			<h3 className="title">{instance.name}</h3>
			<span className="path">{instance.path}</span>
		</div>
	)
}