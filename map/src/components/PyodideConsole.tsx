import { Crosshair, LoaderCircle, Play, RefreshCw, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useMapTray } from "@/cards/MapTrayContext";
import { useManifestEditor } from "@/graphql/useManifestEditor";
import { consoleTextColor } from "@/theme/console";
import { useTheme } from "@/theme/theme";
import {
	PythonCodeEditor,
	type PythonCodeEditorHandle,
} from "./PythonCodeEditor";

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
		y: Math.max(
			8,
			Math.min(position.y, window.innerHeight - bounds.height - 8),
		),
	};
}

export function PyodideConsole({
	isOpen,
	panelWidth,
	onOpenChange,
	onPanelWidthChange,
}: PyodideConsoleProps) {
	const editor = useManifestEditor(isOpen);
	const editorRef = useRef<PythonCodeEditorHandle | null>(null);
	const { selectedBlock } = useMapTray();
	const theme = useTheme();
	const palette = theme.console;
	const [follow, setFollow] = useState(true);
	const [position, setPosition] = useState<PanelPosition>({ x: 16, y: 72 });
	const panelRef = useRef<HTMLElement | null>(null);
	const gesture = useRef<PanelGesture | null>(null);
	const startGesture = (
		event: React.PointerEvent<HTMLElement>,
		kind: PanelGesture["kind"],
	) => {
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
		if (isOpen && follow && selectedBlock?.lineNumber)
			editorRef.current?.moveToLine(selectedBlock.lineNumber);
	}, [isOpen, follow, selectedBlock?.lineNumber]);
	useEffect(() => {
		if (!isOpen) return;
		const move = (event: PointerEvent) => {
			const active = gesture.current;
			if (!active || event.pointerId !== active.pointerId || !panelRef.current)
				return;
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
			style={{
				left: position.x,
				top: position.y,
				width: panelWidth,
				backgroundColor: palette.panel,
				color: consoleTextColor(palette.text),
			}}
			className="fixed z-[920] flex h-[calc(100vh-6rem)] max-h-[46rem] max-w-[calc(100vw-1rem)] overflow-hidden rounded-[28px] border border-current/15 shadow-2xl backdrop-blur-md"
		>
			<div className="flex min-w-0 flex-1 flex-col">
				<header
					className="flex cursor-grab touch-none select-none flex-wrap items-center gap-2 border-b border-current/15 p-3 active:cursor-grabbing"
					title="Drag to move console"
					onPointerDown={(event) => {
						if ((event.target as HTMLElement).closest("button")) return;
						startGesture(event, "drag");
					}}
				>
					<button
						type="button"
						aria-label="Hide terminal file"
						onClick={() => onOpenChange(false)}
					>
						<X size={16} />
					</button>
					<span
						className="min-w-0 flex-1 truncate font-mono text-xs"
						title={editor.doc?.folderPath}
					>
						{editor.doc
							? [editor.doc.folderPath, editor.doc.entrypoint]
									.filter(Boolean)
									.join("/")
							: "index.scry"}
						{editor.dirty ? " *" : ""}
					</span>
					<button
						type="button"
						onClick={() => void editor.run()}
						disabled={
							!editor.doc?.writable || editor.running || editor.conflict
						}
						className="inline-flex items-center gap-1 rounded border border-current/20 px-3 py-1 disabled:opacity-40"
					>
						{editor.running ? (
							<LoaderCircle size={14} className="animate-spin" />
						) : (
							<Play size={14} />
						)}{" "}
						Save and Run
					</button>
				</header>
				{editor.conflict && (
					<div role="alert" className="bg-amber-500/15 p-3 text-sm">
						Source changed elsewhere. Your draft is retained; copy it before
						reloading the current source.
					</div>
				)}
				{editor.doc && !editor.doc.writable && (
					<div role="status" className="p-3 text-sm">
						This source is read-only in the current session.
					</div>
				)}
				<div
					className="min-h-0 flex-1 overflow-hidden"
					style={{ backgroundColor: palette.background }}
				>
					<PythonCodeEditor
						ref={editorRef}
						value={editor.code}
						onChange={editor.setCode}
						theme={theme}
					/>
				</div>
				<footer className="border-t border-current/15 p-3 text-xs">
					<div className="mb-2 flex flex-wrap items-center gap-2">
						{editor.canCancel && (
							<button
								type="button"
								onClick={editor.cancel}
								className="mr-auto inline-flex items-center rounded-lg border border-current/15 bg-white/5 px-2.5 py-1.5 font-medium transition-colors hover:border-current/30 hover:bg-white/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-current"
							>
								Cancel run
							</button>
						)}
						<div className="ml-auto flex flex-wrap items-center justify-end gap-2">
							<label
								className="inline-flex cursor-pointer items-center gap-1.5 rounded-lg border border-current/20 px-2.5 py-1.5 font-medium shadow-sm transition-colors focus-within:ring-2 focus-within:ring-current"
								style={{
									color: consoleTextColor(
										follow ? palette.accent : palette.text,
									),
									backgroundColor: follow
										? palette.selection
										: palette.activeLine,
								}}
							>
								<input
									type="checkbox"
									checked={follow}
									onChange={(event) => setFollow(event.target.checked)}
									className="sr-only"
								/>
								<Crosshair size={14} strokeWidth={2} aria-hidden="true" />
								Follow selected block
							</label>
							<button
								type="button"
								disabled={editor.running}
								onClick={() => {
									if (
										!editor.dirty ||
										window.confirm(
											"Discard this unsaved draft and reload the saved source?",
										)
									)
										void editor.reload();
								}}
								className="inline-flex items-center gap-1.5 rounded-lg border border-current/15 bg-white/5 px-2.5 py-1.5 font-medium shadow-sm transition-colors hover:border-current/30 hover:bg-white/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-current disabled:cursor-not-allowed disabled:opacity-40"
							>
								<RefreshCw size={14} strokeWidth={2} aria-hidden="true" />
								Reload source
							</button>
						</div>
					</div>
					<div role="status" aria-live="polite">
						{editor.status}
					</div>
					{editor.output && (
						<pre
							role="log"
							aria-label="Python output"
							className="mt-2 max-h-36 overflow-auto whitespace-pre-wrap"
						>
							{editor.output}
						</pre>
					)}
				</footer>
			</div>
			<button
				type="button"
				aria-label="Resize terminal panel"
				className="w-3 shrink-0 touch-none cursor-col-resize hover:bg-white/10"
				onPointerDown={(event) => startGesture(event, "resize")}
			/>
		</section>
	);
}
