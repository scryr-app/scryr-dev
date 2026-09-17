import { PencilLine } from "lucide-react";
import { DiagramButton } from "@/components/DiagramButton";

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
			<div
				className="fixed left-4 top-4 z-[1000]"
				style={{ pointerEvents: "auto" }}
			>
				<div className="flex items-center rounded-full border border-white/15 bg-black/40">
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
						className="mr-2 flex size-9 shrink-0 items-center justify-center rounded-full text-white/60 transition-colors hover:bg-white/15 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/60 aria-pressed:bg-white/15 aria-pressed:text-white"
					>
						<PencilLine size={18} aria-hidden="true" />
					</button>
				</div>
			</div>

			{/* Top-right — Account */}
			<div
				className="fixed right-4 top-4 z-[900]"
				style={{ pointerEvents: "auto" }}
			></div>
		</>
	);
}
