import { PencilLine } from "lucide-react";
import { DiagramButton } from "@/components/DiagramButton";
import styles from "./TopCornerButtons.module.css";

interface TopCornerButtonsProps {
	isPyodideOpen: boolean;
	onTogglePyodide: () => void;
}

export function TopCornerButtons({
	isPyodideOpen,
	onTogglePyodide,
}: TopCornerButtonsProps) {
	return (
		<>
			{/* Top-left — Diagram */}
			<div className={styles.diagramPosition} style={{ pointerEvents: "auto" }}>
				<div className={styles.diagramControls}>
					<DiagramButton />
					<button
						type="button"
						onClick={onTogglePyodide}
						aria-label={
							isPyodideOpen ? "Hide diagram editor" : "Show diagram editor"
						}
						aria-pressed={isPyodideOpen}
						title={
							isPyodideOpen ? "Hide diagram editor" : "Show diagram editor"
						}
						className={styles.editorButton}
					>
						<PencilLine size={18} aria-hidden="true" />
					</button>
				</div>
			</div>

			{/* Top-right — Account */}
			<div
				className={styles.accountPosition}
				style={{ pointerEvents: "auto" }}
			></div>
		</>
	);
}
