import { DiagramButton } from "@/components/DiagramButton";
import { TerminalLauncherButton } from "@/components/PyodideConsole";

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
				className="fixed left-4 top-4 z-[900]"
				style={{ pointerEvents: "auto" }}
			>
				<div className="flex items-center gap-2 rounded-full border border-white/15 bg-black/40 pr-1">
					<DiagramButton />
					<TerminalLauncherButton
						isOpen={isPyodideOpen}
						onToggle={onTogglePyodide}
					/>
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
