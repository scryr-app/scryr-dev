import { LoaderCircle, Play, RefreshCw, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useManifestEditor } from "@/graphql/useManifestEditor";
import { consoleTextColor } from "@/theme/console";
import { useTheme } from "@/theme/theme";
import styles from "./PyodideConsole.module.css";
import { PythonCodeEditor } from "./PythonCodeEditor";

interface PyodideConsoleProps {
	isOpen: boolean;
	panelWidth: number;
	onOpenChange: (isOpen: boolean) => void;
	onPanelWidthChange: (width: number) => void;
}

type PanelPosition = { x: number; y: number };
type PanelGesture = {
	pointerId: number;
	x: number;
	y: number;
	left: number;
	top: number;
	width: number;
	kind: "drag" | "resize";
	cursor: string;
	userSelect: string;
};

function constrainPosition(position: PanelPosition, panel: HTMLElement) {
	const bounds = panel.getBoundingClientRect();
	return {
		x: Math.max(8, Math.min(position.x, window.innerWidth - bounds.width - 8)),
		y: Math.max(8, Math.min(position.y, window.innerHeight - bounds.height - 8)),
	};
}

export function PyodideConsole({
	isOpen,
	panelWidth,
	onOpenChange,
	onPanelWidthChange,
}: PyodideConsoleProps) {
	const editor = useManifestEditor(isOpen);
	const theme = useTheme();
	const palette = theme.console;
	const [position, setPosition] = useState<PanelPosition>({ x: 16, y: 72 });
	const panelRef = useRef<HTMLElement | null>(null);
	const gesture = useRef<PanelGesture | null>(null);
	const startGesture = (event: React.PointerEvent<HTMLElement>, kind: PanelGesture["kind"]) => {
		if (event.button !== 0 || gesture.current) return;
		event.preventDefault();
		event.currentTarget.setPointerCapture(event.pointerId);
		gesture.current = {
			pointerId: event.pointerId,
			x: event.clientX,
			y: event.clientY,
			left: position.x,
			top: position.y,
			width: panelRef.current?.getBoundingClientRect().width ?? panelWidth,
			kind,
			cursor: document.body.style.cursor,
			userSelect: document.body.style.userSelect,
		};
		document.body.style.cursor = kind === "drag" ? "grabbing" : "col-resize";
		document.body.style.userSelect = "none";
	};
	useEffect(() => {
		if (!isOpen) return;
		const move = (event: PointerEvent) => {
			const active = gesture.current;
			if (!active || event.pointerId !== active.pointerId || !panelRef.current) return;
			if (active.kind === "drag") {
				setPosition(
					constrainPosition(
						{
							x: active.left + event.clientX - active.x,
							y: active.top + event.clientY - active.y,
						},
						panelRef.current,
					),
				);
			} else {
				onPanelWidthChange(
					Math.min(
						window.innerWidth - active.left - 8,
						Math.max(320, active.width + event.clientX - active.x),
					),
				);
			}
		};
		const stop = () => {
			if (!gesture.current) return;
			document.body.style.cursor = gesture.current.cursor;
			document.body.style.userSelect = gesture.current.userSelect;
			gesture.current = null;
		};
		window.addEventListener("pointermove", move);
		window.addEventListener("pointerup", stop);
		window.addEventListener("pointercancel", stop);
		window.addEventListener("blur", stop);
		return () => {
			window.removeEventListener("pointermove", move);
			window.removeEventListener("pointerup", stop);
			window.removeEventListener("pointercancel", stop);
			window.removeEventListener("blur", stop);
			stop();
		};
	}, [onPanelWidthChange, isOpen]);
	useEffect(() => {
		if (!isOpen) return;
		const keepVisible = () => {
			const panel = panelRef.current;
			if (panel) setPosition((current) => constrainPosition(current, panel));
		};
		keepVisible();
		window.addEventListener("resize", keepVisible);
		return () => window.removeEventListener("resize", keepVisible);
	}, [isOpen]);
	if (!isOpen) return null;
	return (
		<section
			ref={panelRef}
			aria-label="Manifest source editor"
			data-camera-occluder
			style={{
				left: position.x,
				top: position.y,
				width: panelWidth,
				backgroundColor: palette.panel,
				color: consoleTextColor(palette.text),
			}}
			className={styles.panel}
		>
			<div className={styles.content}>
				<header
					className={styles.header}
					title="Drag to move console"
					onPointerDown={(event) => {
						if ((event.target as HTMLElement).closest("button")) return;
						startGesture(event, "drag");
					}}
				>
					<button type="button" aria-label="Hide terminal file" onClick={() => onOpenChange(false)}>
						<X size={16} />
					</button>
					<span className={styles.filename} title={editor.doc?.folderPath}>
						{editor.doc
							? [editor.doc.folderPath, editor.doc.entrypoint].filter(Boolean).join("/")
							: "index.scry"}
						{editor.dirty ? " *" : ""}
					</span>
				</header>
				{editor.conflict && (
					<div role="alert" className={styles.conflict}>
						Source changed elsewhere. Your draft is retained; copy it before reloading the current
						source.
					</div>
				)}
				{editor.doc && !editor.doc.writable && (
					<div role="status" className={styles.notice}>
						This source is read-only in the current session.
					</div>
				)}
				<div className={styles.editor} style={{ backgroundColor: palette.background }}>
					<PythonCodeEditor value={editor.code} onChange={editor.setCode} theme={theme} />
				</div>
				<footer className={styles.footer}>
					<div className={styles.actions}>
						{editor.canCancel && (
							<button type="button" onClick={editor.cancel} className={styles.cancelButton}>
								Cancel run
							</button>
						)}
						<div className={styles.primaryActions}>
							<button
								type="button"
								disabled={editor.running}
								onClick={() => {
									if (
										!editor.dirty ||
										window.confirm("Discard this unsaved draft and reload the saved source?")
									)
										void editor.reload();
								}}
								className={styles.reloadButton}
							>
								<RefreshCw size={14} strokeWidth={2} aria-hidden="true" />
								Reload source
							</button>
							<button
								type="button"
								onClick={() => void editor.run()}
								disabled={!editor.doc?.writable || editor.running || editor.conflict}
								className={styles.runButton}
								style={{
									backgroundColor: palette.accent,
									color: consoleTextColor(palette.background),
								}}
							>
								{editor.running ? (
									<LoaderCircle size={14} className={styles.spinner} />
								) : (
									<Play size={14} />
								)}{" "}
								Save and Run
							</button>
						</div>
					</div>
					<div role="status" aria-live="polite">
						{editor.status}
					</div>
					{editor.output && (
						<pre role="log" aria-label="Python output" className={styles.output}>
							{editor.output}
						</pre>
					)}
				</footer>
			</div>
			<button
				type="button"
				aria-label="Resize terminal panel"
				className={styles.resizeHandle}
				onPointerDown={(event) => startGesture(event, "resize")}
			/>
		</section>
	);
}
