import { Crosshair, LoaderCircle, Play, RefreshCw, X } from "lucide-react";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { useMapTray } from "@/cards/MapTrayContext";
import { useManifestEditor } from "@/graphql/useManifestEditor";
import { getDiagramMode, subscribeDiagramMode } from "@/theme/theme";
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
export function PyodideConsole({
	isOpen,
	panelWidth,
	onOpenChange,
	onPanelWidthChange,
}: PyodideConsoleProps) {
	const editor = useManifestEditor(isOpen);
	const editorRef = useRef<PythonCodeEditorHandle | null>(null);
	const { selectedBlock } = useMapTray();
	const theme = useSyncExternalStore(subscribeDiagramMode, getDiagramMode);
	const [follow, setFollow] = useState(true);
	const resize = useRef<{ x: number; width: number } | null>(null);
	useEffect(() => {
		if (isOpen && follow && selectedBlock?.lineNumber)
			editorRef.current?.moveToLine(selectedBlock.lineNumber);
	}, [isOpen, follow, selectedBlock?.lineNumber]);
	useEffect(() => {
		const move = (event: PointerEvent) => {
			if (resize.current)
				onPanelWidthChange(
					Math.max(
						320,
						Math.min(
							window.innerWidth - 80,
							resize.current.width + event.clientX - resize.current.x,
						),
					),
				);
		};
		const stop = () => {
			resize.current = null;
			document.body.style.cursor = "";
			document.body.style.userSelect = "";
		};
		window.addEventListener("pointermove", move);
		window.addEventListener("pointerup", stop);
		return () => {
			window.removeEventListener("pointermove", move);
			window.removeEventListener("pointerup", stop);
			stop();
		};
	}, [onPanelWidthChange]);
	if (!isOpen) return null;
	return (
		<section
			aria-label="Manifest source editor"
			style={{ width: panelWidth }}
			className="absolute left-4 top-18 z-[920] flex h-[calc(100vh-6rem)] max-h-[46rem] overflow-hidden rounded-[28px] border border-white/15 bg-black/40 text-slate-100 shadow-2xl backdrop-blur-md"
		>
			<div className="flex min-w-0 flex-1 flex-col">
				<header className="flex flex-wrap items-center gap-2 border-b border-current/15 p-3">
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
					className={`min-h-0 flex-1 overflow-hidden ${theme === "dark" ? "bg-[#1e1e1e]" : "bg-white"}`}
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
								className="mr-auto inline-flex items-center rounded-lg border border-white/10 bg-white/5 px-2.5 py-1.5 font-medium text-slate-300 transition-colors hover:border-white/20 hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400/70"
							>
								Cancel run
							</button>
						)}
						<div className="ml-auto flex flex-wrap items-center justify-end gap-2">
							<label
								className={`inline-flex cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 py-1.5 font-medium shadow-sm transition-colors focus-within:ring-2 focus-within:ring-cyan-400/70 ${
									follow
										? "border-cyan-300/25 bg-cyan-400/15 text-cyan-100 hover:bg-cyan-400/20"
										: "border-white/10 bg-white/5 text-slate-300 hover:border-white/20 hover:bg-white/10 hover:text-white"
								}`}
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
								className="inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/5 px-2.5 py-1.5 font-medium text-slate-300 shadow-sm transition-colors hover:border-white/20 hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400/70 disabled:cursor-not-allowed disabled:opacity-40"
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
				className="w-3 shrink-0 cursor-col-resize hover:bg-white/10"
				onPointerDown={(event) => {
					resize.current = { x: event.clientX, width: panelWidth };
					document.body.style.cursor = "col-resize";
					document.body.style.userSelect = "none";
				}}
			/>
		</section>
	);
}
