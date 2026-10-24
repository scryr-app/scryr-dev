import { useEffect, useState } from "react";
import styles from "./HtmlGraphCard.module.css";

export function HtmlGraphContent({ mode, onFocus }: { mode: string; onFocus: () => void }) {
	const [live, setLive] = useState(true);
	const [tick, setTick] = useState(0);
	const [load, setLoad] = useState(55);
	const [selected, setSelected] = useState(5);
	const [note, setNote] = useState("");
	useEffect(() => {
		if (!live) return;
		const timer = window.setInterval(() => setTick((value) => value + 1), 700);
		return () => window.clearInterval(timer);
	}, [live]);
	const values = Array.from({ length: 16 }, (_, i) =>
		Math.round(25 + load * 0.6 + Math.sin(i * 0.8 + tick * 0.3) * 18),
	);
	return (
		<section className={styles.content} aria-label="Interactive graph prototype">
			<header className={styles.header}>
				<div>
					<small>DEMO DATA · {mode}</small>
					<h2>Service activity</h2>
				</div>
				<button type="button" onClick={onFocus}>
					Enlarge card
				</button>
			</header>
			<div className={styles.metrics}>
				<div>
					<small>Selected sample</small>
					<strong>{values[selected]} ms</strong>
				</div>
				<div>
					<small>Traffic</small>
					<strong>{load}%</strong>
				</div>
				<button type="button" aria-pressed={live} onClick={() => setLive(!live)}>
					{live ? "Pause updates" : "Resume updates"}
				</button>
			</div>
			<svg
				className={styles.chart}
				viewBox="0 0 600 150"
				role="img"
				aria-label="Simulated latency over sixteen samples"
			>
				{[30, 70, 110].map((y) => (
					<line key={y} x1="10" y1={y} x2="590" y2={y} stroke="#334155" />
				))}
				<polyline
					fill="none"
					stroke="#5eead4"
					strokeWidth="3"
					points={values.map((v, i) => `${15 + i * 38},${145 - v * 1.3}`).join(" ")}
				/>
				{values.map((v, i) => (
					<circle
						key={i}
						cx={15 + i * 38}
						cy={145 - v * 1.3}
						r={i === selected ? 7 : 4}
						fill={i === selected ? "#fbbf24" : "#5eead4"}
					/>
				))}
			</svg>
			<div className={styles.controls}>
				<label>
					Traffic
					<input
						aria-label="Traffic"
						type="range"
						min="0"
						max="100"
						value={load}
						onChange={(event) => setLoad(Number(event.target.value))}
					/>
				</label>
				<label>
					Inspect sample
					<select
						aria-label="Inspect sample"
						value={selected}
						onChange={(event) => setSelected(Number(event.target.value))}
					>
						{values.map((_, i) => (
							<option key={i} value={i}>
								Sample {i + 1}
							</option>
						))}
					</select>
				</label>
			</div>
			<label className={styles.note}>
				Card notes
				<input
					placeholder="Type here to test keyboard focus"
					value={note}
					onChange={(event) => setNote(event.target.value)}
				/>
			</label>
			<div className={styles.table} tabIndex={0} aria-label="Scrollable sample values">
				<table>
					<thead>
						<tr>
							<th>Sample</th>
							<th>Latency</th>
							<th>Status</th>
						</tr>
					</thead>
					<tbody>
						{values.map((value, i) => (
							<tr key={i}>
								<td>{i + 1}</td>
								<td>{value} ms</td>
								<td>{value > 80 ? "Elevated" : "Healthy"}</td>
							</tr>
						))}
					</tbody>
				</table>
			</div>
		</section>
	);
}
