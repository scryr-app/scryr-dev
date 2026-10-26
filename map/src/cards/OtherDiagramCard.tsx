import { Layers } from "lucide-react";
import styles from "./CardContent.module.css";

export interface OtherDiagramCardProps {
	/** Names of the diagrams (regions/tags) this block belongs to. */
	diagrams?: string[];
}

export function OtherDiagramCard({ diagrams = [] }: OtherDiagramCardProps) {
	const names = [...new Set(diagrams.filter((name) => name.trim()))];
	return (
		<article className={styles.card} aria-label="Other diagrams">
			<header className={styles.header}>
				<h2>
					<Layers aria-hidden="true" />
					Other diagrams
				</h2>
			</header>
			{names.length === 0 ? (
				<p className={styles.empty}>No diagrams found.</p>
			) : (
				<ul className={styles.diagrams}>
					{names.map((name) => (
						<li className={styles.panel} key={name}>
							<span className={styles.caption}>Diagram</span>
							<span>{name}</span>
						</li>
					))}
				</ul>
			)}
		</article>
	);
}
