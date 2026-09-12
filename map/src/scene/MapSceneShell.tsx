import type { ReactNode } from "react";
import { useState } from "react";
import { MapTray } from "@/cards/MapTray";
import { MapTrayProvider } from "@/cards/MapTrayContext";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { PyodideConsole } from "@/components/PyodideConsole";
import { TopCornerButtons } from "@/components/TopCornerButtons";
import { useTheme } from "@/theme/theme";
import { MapDiagram } from "./MapDiagram";

interface MapSceneShellProps {
	header?: ReactNode;
}

export function MapSceneShell({ header }: MapSceneShellProps) {
	const theme = useTheme();
	const [isPyodideOpen, setIsPyodideOpen] = useState(true);
	const [pyodidePanelWidth, setPyodidePanelWidth] = useState(() => {
		if (typeof window === "undefined") {
			return 420;
		}
		return Math.round(window.innerWidth * 0.3);
	});

	const togglePyodide = () => {
		setIsPyodideOpen((value) => !value);
	};

	return (
		<MapTrayProvider>
			<div
				style={{
					width: "100vw",
					height: "100vh",
					background: theme.background,
				}}
			>
				<ErrorBoundary name="PyodideConsole" fallback={null}>
					<PyodideConsole
						isOpen={isPyodideOpen}
						panelWidth={pyodidePanelWidth}
						onOpenChange={setIsPyodideOpen}
						onPanelWidthChange={setPyodidePanelWidth}
					/>
				</ErrorBoundary>
				<div className="relative min-w-0 h-full">
					{header}
					<MapDiagram key={theme.id} />
					<ErrorBoundary name="MapTray" fallback={null}>
						<MapTray
							isPyodideOpen={isPyodideOpen}
							onTogglePyodide={togglePyodide}
						/>
					</ErrorBoundary>
					<ErrorBoundary name="TopCornerButtons" fallback={null}>
						<TopCornerButtons />
					</ErrorBoundary>
				</div>
			</div>
		</MapTrayProvider>
	);
}
