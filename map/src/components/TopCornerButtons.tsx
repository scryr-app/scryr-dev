import { DiagramButton } from "@/components/DiagramButton";

export function TopCornerButtons() {
	return (
		<>
			{/* Top-left — Diagram */}
			<div
				className="fixed left-4 top-4 z-[900]"
				style={{ pointerEvents: "auto" }}
			>
				<div className="flex items-center rounded-full border border-white/15 bg-black/40">
					<DiagramButton />
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
