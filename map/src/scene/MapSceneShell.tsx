import type { ReactNode } from "react";
import { useState } from "react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { PyodideConsole } from "@/components/PyodideConsole";
import { TopCornerButtons } from "@/components/TopCornerButtons";
import { useTheme } from "@/theme/theme";
import { MapDiagram } from "./MapDiagram";
import styles from "./MapSceneShell.module.css";
import { MapToolbar } from "./MapToolbar";

interface MapSceneShellProps {
	header?: ReactNode;
}

export function MapSceneShell({ header }: MapSceneShellProps) {
	const theme = useTheme();
	const [isPyodideOpen, setIsPyodideOpen] = useState(true);
	const [pyodidePanelWidth, setPyodidePanelWidth] = useState(() => {
		if (typeof window === "undefined") {
			return 480;
		}
		return Math.max(480, Math.round(window.innerWidth * 0.3));
	});

	const togglePyodide = () => {
		setIsPyodideOpen((value) => !value);
	};

	return (
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
			<div className={styles.scene}>
				{header}
				<MapDiagram key={theme.id} />
				<ErrorBoundary name="MapToolbar" fallback={null}>
					<MapToolbar
						isPyodideOpen={isPyodideOpen}
						onTogglePyodide={togglePyodide}
					/>
				</ErrorBoundary>
				<ErrorBoundary name="TopCornerButtons" fallback={null}>
					<TopCornerButtons
						isPyodideOpen={isPyodideOpen}
						onTogglePyodide={togglePyodide}
					/>
				</ErrorBoundary>
			</div>
		</div>
	);
}
