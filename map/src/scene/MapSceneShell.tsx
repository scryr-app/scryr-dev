import type { ReactNode } from "react";
import { useState } from "react";
import { useCardFocus } from "@/cards/CardFocusContext";
import { CardFocusView } from "@/cards/CardFocusView";
import { MapTray } from "@/cards/MapTray";
import { MapTrayProvider } from "@/cards/MapTrayContext";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { PyodideConsole } from "@/components/PyodideConsole";
import { TopCornerButtons } from "@/components/TopCornerButtons";
import { useTheme } from "@/theme/theme";
import { MapDiagram } from "./MapDiagram";
import styles from "./MapSceneShell.module.css";

interface MapSceneShellProps {
	header?: ReactNode;
}

export function MapSceneShell(props: MapSceneShellProps) {
	return (
		<MapTrayProvider>
			<MapSceneContent {...props} />
		</MapTrayProvider>
	);
}

function MapSceneContent({ header }: MapSceneShellProps) {
	const { focusedIds } = useCardFocus();
	const isFocused = focusedIds.length > 0;
	const chromeStyle = { display: isFocused ? "none" : "contents" };
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
		<>
			<div
				style={{
					width: "100vw",
					height: "100vh",
					background: theme.background,
				}}
			>
				<div style={chromeStyle}>
					<ErrorBoundary name="PyodideConsole" fallback={null}>
						<PyodideConsole
							isOpen={isPyodideOpen}
							panelWidth={pyodidePanelWidth}
							onOpenChange={setIsPyodideOpen}
							onPanelWidthChange={setPyodidePanelWidth}
						/>
					</ErrorBoundary>
				</div>
				<div className={styles.scene}>
					<div style={chromeStyle}>{header}</div>
					<MapDiagram key={theme.id} />
					<div style={chromeStyle}>
						<ErrorBoundary name="MapTray" fallback={null}>
							<MapTray isPyodideOpen={isPyodideOpen} onTogglePyodide={togglePyodide} />
						</ErrorBoundary>
						<ErrorBoundary name="TopCornerButtons" fallback={null}>
							<TopCornerButtons isPyodideOpen={isPyodideOpen} onTogglePyodide={togglePyodide} />
						</ErrorBoundary>
					</div>
				</div>
			</div>
			<CardFocusView />
		</>
	);
}
