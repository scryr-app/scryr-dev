import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
	LoaderCircle,
	LocateFixed,
	Moon,
	PencilLine,
	Play,
	Sun,
	X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useMapTray } from "@/cards/MapTrayContext";
import {
	PythonCodeEditor,
	type PythonCodeEditorHandle,
} from "@/components/PythonCodeEditor";
import { graphqlFetcher } from "@/graphql/client";
import {
	ArtifactKind,
	GetBlocksDocument,
	type GetBlocksQuery,
	type GetBlocksQueryVariables,
	type UpsertGeneratedManifestInput,
	useGetBlocksQuery,
	useGetScryrMapsQuery,
} from "@/graphql/generated";
import { clearRuntimePreviewBlocks } from "@/graphql/runtimePreviewStore";
import { mapSelectionStore, useSample } from "@/graphql/sampleStore";
import { runPythonSample } from "@/pyodide/pyodideRuntime";
import { sampleMernPython } from "@/pyodide/sdkSources";

const MIN_PANEL_WIDTH = 320;
const SCHEMA_ARTIFACT_KEY = "";
const UPSERT_GENERATED_MANIFEST_MUTATION = /* GraphQL */ `
	mutation UpsertGeneratedManifest($input: UpsertGeneratedManifestInput!) {
		upsertGeneratedManifest(input: $input) {
			id
		}
	}
`;

interface PyodideConsoleProps {
	isOpen: boolean;
	panelWidth: number;
	onOpenChange: (isOpen: boolean) => void;
	onPanelWidthChange: (width: number) => void;
}

interface TerminalLauncherButtonProps {
	isOpen: boolean;
	onToggle: () => void;
}

export function TerminalLauncherButton({
	isOpen,
	onToggle,
}: TerminalLauncherButtonProps) {
	return (
		<button
			type="button"
			aria-label={isOpen ? "Hide terminal panel" : "Show terminal panel"}
			onClick={onToggle}
			className="rounded-full border border-white/12 bg-white/6 p-1.5 text-white/55 transition hover:bg-white/10 hover:text-white/85"
		>
			<PencilLine size={13} strokeWidth={2.2} />
		</button>
	);
}

export function PyodideConsole({
	isOpen,
	panelWidth,
	onOpenChange,
	onPanelWidthChange,
}: PyodideConsoleProps) {
	const sample = useSample();
	const queryClient = useQueryClient();
	const [code, setCode] = useState(sampleMernPython);
	const [_status, setStatus] = useState("Idle");
	const [isRunning, setIsRunning] = useState(false);
	const [, setOutput] = useState("");
	const [theme, setTheme] = useState<"dark" | "light">("dark");
	const [followSelectedBlock, setFollowSelectedBlock] = useState(true);
	const resizeStateRef = useRef<{ startX: number; startWidth: number } | null>(
		null,
	);
	const editorRef = useRef<PythonCodeEditorHandle | null>(null);
	const { selectedBlock } = useMapTray();

	const isDark = theme === "dark";
	const panelClassName = isDark
		? "border-white/18 bg-black/78 text-slate-100 shadow-black/50"
		: "border-black/10 bg-white/42 text-slate-900 shadow-slate-950/10";
	const headerClassName = isDark
		? "border-white/12 bg-black/34"
		: "border-black/8 bg-white/18";
	const chromeButtonClassName = isDark
		? "border-white/14 bg-white/10 text-white/72 hover:bg-white/16 hover:text-white"
		: "border-black/10 bg-black/4 text-slate-700 hover:bg-black/8 hover:text-slate-950";
	const runButtonClassName = isDark
		? "border-white/14 bg-white/12 text-white/92 hover:bg-white/18"
		: "border-black/10 bg-black/5 text-slate-800 hover:bg-black/10";
	const editorSurfaceClassName = isDark
		? "bg-transparent text-slate-100"
		: "bg-transparent text-slate-900";
	const resizeLineClassName = isDark
		? "bg-white/14 group-hover:bg-white/26"
		: "bg-black/10 group-hover:bg-black/22";
	const persistGeneratedManifest = useMutation({
		mutationFn: async (input: UpsertGeneratedManifestInput) => {
			return await graphqlFetcher<
				{ upsertGeneratedManifest: { id: string } },
				{ input: UpsertGeneratedManifestInput }
			>(UPSERT_GENERATED_MANIFEST_MUTATION, { input })();
		},
	});

	useEffect(() => {
		let cancelled = false;

		const boot = async () => {
			setStatus("Loading Pyodide runtime");
			try {
				const result = await runPythonSample(sampleMernPython);
				if (cancelled) {
					return;
				}
				setStatus(
					`Loaded ${result.manifests.length} manifests from mern/index.scry`,
				);
				setOutput(result.stdout || JSON.stringify(result.manifests, null, 2));
			} catch (error) {
				if (cancelled) {
					return;
				}
				setStatus("Pyodide bootstrap failed");
				setOutput(error instanceof Error ? error.message : String(error));
			}
		};

		void boot();

		return () => {
			cancelled = true;
		};
	}, []);

	const runCode = async () => {
		setIsRunning(true);
		setStatus(`Running ${sample}/index.scry`);
		setOutput("");

		try {
			const result = await runPythonSample(code, (line) => {
				setOutput((current) => `${current}${line}\n`);
			});
			const lineNumbers = inferManifestLineNumbers(code);
			if (result.schema) {
				setStatus("Persisting generated manifest schema");
				await persistGeneratedManifest.mutateAsync({
					artifactKind: ArtifactKind.Schema,
					artifactKey: SCHEMA_ARTIFACT_KEY,
					folderPath: sample,
					fileName: "index.scry",
					content: JSON.stringify(result.schema, null, 2),
				});
			}
			setStatus("Persisting generated manifests");
			const diagram = result.diagrams.at(0);
			const scryIdentifier = diagram?.name ?? sample;
			const mapName =
				typeof diagram?.diagram.name === "string"
					? diagram.diagram.name
					: scryIdentifier;
			await persistGeneratedManifest.mutateAsync({
				artifactKind: ArtifactKind.Value,
				artifactKey: scryIdentifier,
				folderPath: sample,
				fileName: "index.scry",
				scryIdentifier,
				name: mapName,
				content: JSON.stringify(
					{
						files: [
							{
								path: "index.scry",
								content: code,
							},
						],
						manifests: result.manifests.map((item) => ({
							...item.manifest,
							line_number: lineNumbers.get(item.name) ?? null,
							variable_name: item.name,
						})),
						diagrams: result.diagrams.map((item) => ({
							...item.diagram,
							line_number: lineNumbers.get(item.name) ?? null,
							variable_name: item.name,
						})),
					},
					null,
					2,
				),
			});
			mapSelectionStore.set({ id: scryIdentifier, key: scryIdentifier });
			clearRuntimePreviewBlocks();
			setStatus("Refreshing block display");
			await queryClient.invalidateQueries({
				queryKey: useGetScryrMapsQuery.getKey(),
			});
			await queryClient.fetchQuery<
				GetBlocksQuery,
				Error,
				GetBlocksQuery,
				ReturnType<typeof useGetBlocksQuery.getKey>
			>({
				queryKey: useGetBlocksQuery.getKey({ scryIdentifier }),
				queryFn: graphqlFetcher<GetBlocksQuery, GetBlocksQueryVariables>(
					GetBlocksDocument,
					{ scryIdentifier },
				),
			});
			setStatus(
				`Saved ${result.manifests.length} manifest objects and refreshed blocks`,
			);
			setOutput(
				(current) => current || JSON.stringify(result.manifests, null, 2),
			);
		} catch (error) {
			setStatus("Execution failed");
			setOutput(error instanceof Error ? error.message : String(error));
		} finally {
			setIsRunning(false);
		}
	};

	useEffect(() => {
		if (!isOpen) {
			return;
		}

		const handlePointerMove = (event: PointerEvent) => {
			const resizeState = resizeStateRef.current;
			if (!resizeState) {
				return;
			}

			const nextWidth =
				resizeState.startWidth + event.clientX - resizeState.startX;
			const maxWidth = Math.max(MIN_PANEL_WIDTH, window.innerWidth - 360);
			onPanelWidthChange(
				Math.min(Math.max(nextWidth, MIN_PANEL_WIDTH), maxWidth),
			);
		};

		const handlePointerUp = () => {
			resizeStateRef.current = null;
			document.body.style.cursor = "";
			document.body.style.userSelect = "";
		};

		window.addEventListener("pointermove", handlePointerMove);
		window.addEventListener("pointerup", handlePointerUp);

		return () => {
			window.removeEventListener("pointermove", handlePointerMove);
			window.removeEventListener("pointerup", handlePointerUp);
		};
	}, [isOpen, onPanelWidthChange]);

	useEffect(() => {
		if (!isOpen || !followSelectedBlock) {
			return;
		}

		const lineNumber = selectedBlock?.lineNumber;
		if (!lineNumber || lineNumber < 1) {
			return;
		}

		editorRef.current?.moveToLine(lineNumber);
	}, [followSelectedBlock, isOpen, selectedBlock?.lineNumber]);

	if (!isOpen) return null;

	return (
		<section
			className={`absolute left-4 top-18 z-[920] flex h-[calc(100vh-2rem)] max-h-[46rem] overflow-hidden rounded-[28px] border shadow-2xl backdrop-blur-2xl ${panelClassName}`}
			style={{ width: `${panelWidth}px` }}
		>
			<div className="flex min-w-0 flex-1 flex-col">
				<div
					className={`flex items-center justify-between border-b px-4 py-3 ${headerClassName}`}
				>
					<div className="flex items-center gap-3">
						<button
							type="button"
							aria-label="Hide terminal file"
							onClick={() => {
								onOpenChange(false);
							}}
							className={`rounded-full border p-2 transition ${chromeButtonClassName}`}
						>
							<X size={16} strokeWidth={2.4} />
						</button>
						<div
							className={`rounded-full border px-3 py-1.5 font-mono text-[13px] ${chromeButtonClassName}`}
						>
							{`${sample}/index.scry`}
						</div>
					</div>
					<div className="flex items-center gap-2">
						<div
							className={`flex items-center rounded-full border p-1 ${chromeButtonClassName}`}
						>
							<button
								type="button"
								aria-label={
									followSelectedBlock
										? "Stop following selected blocks in the editor"
										: "Follow selected blocks in the editor"
								}
								onClick={() => {
									setFollowSelectedBlock((current) => !current);
								}}
								className={`rounded-full p-1.5 transition ${
									followSelectedBlock
										? isDark
											? "bg-white/14 text-white"
											: "bg-black/10 text-slate-950"
										: ""
								}`}
							>
								<LocateFixed size={14} strokeWidth={2.2} />
							</button>
							<button
								type="button"
								aria-label="Use light terminal theme"
								onClick={() => {
									setTheme("light");
								}}
								className={`rounded-full p-1.5 transition ${
									theme === "light"
										? isDark
											? "bg-white/14 text-white"
											: "bg-black/10 text-slate-950"
										: ""
								}`}
							>
								<Sun size={14} strokeWidth={2.2} />
							</button>
							<button
								type="button"
								aria-label="Use dark terminal theme"
								onClick={() => {
									setTheme("dark");
								}}
								className={`rounded-full p-1.5 transition ${
									theme === "dark"
										? isDark
											? "bg-white/14 text-white"
											: "bg-black/10 text-slate-950"
										: ""
								}`}
							>
								<Moon size={14} strokeWidth={2.2} />
							</button>
						</div>
						<button
							type="button"
							onClick={() => {
								void runCode();
							}}
							disabled={isRunning || persistGeneratedManifest.isPending}
							className={`inline-flex items-center gap-2 rounded-full border px-4 py-2 text-sm font-medium transition disabled:cursor-wait disabled:opacity-70 ${runButtonClassName}`}
						>
							{isRunning || persistGeneratedManifest.isPending ? (
								<LoaderCircle size={16} className="animate-spin" />
							) : (
								<Play size={16} />
							)}
							Run
						</button>
					</div>
				</div>
				<div className="flex min-h-0 flex-1 overflow-hidden">
					<div
						className={`min-h-0 flex-1 overflow-hidden ${editorSurfaceClassName}`}
					>
						<PythonCodeEditor
							ref={editorRef}
							value={code}
							onChange={setCode}
							theme={theme}
						/>
					</div>
				</div>
			</div>
			<button
				type="button"
				aria-label="Resize terminal panel"
				onPointerDown={(event) => {
					resizeStateRef.current = {
						startX: event.clientX,
						startWidth: panelWidth,
					};
					document.body.style.cursor = "col-resize";
					document.body.style.userSelect = "none";
				}}
				className="group relative w-3 shrink-0 cursor-col-resize bg-transparent"
			>
				<div
					className={`absolute inset-y-6 left-1/2 w-px -translate-x-1/2 transition ${resizeLineClassName}`}
				/>
			</button>
		</section>
	);
}

function inferManifestLineNumbers(code: string) {
	const lineNumbers = new Map<string, number>();

	for (const [index, line] of code.split("\n").entries()) {
		const match = line.match(
			/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:Diagram|Manifest)\s*\(/,
		);
		if (match) {
			lineNumbers.set(match[1], index + 1);
		}
	}

	return lineNumbers;
}
