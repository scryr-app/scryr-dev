import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createContext, useContext, useEffect, useRef, useState } from "react";
import { runPythonDocument } from "@/pyodide/pyodideRuntime";
import { graphqlFetcher } from "./client";
import {
	GetBlocksDocument,
	type GetBlocksQuery,
	type GetBlocksQueryVariables,
	useGetBlocksQuery,
} from "./generated";
import {
	documentKey,
	type ManifestDocument,
	readDocument,
	saveDocument,
} from "./manifestDocument";
import { mapSelectionStore, useSelectedMap } from "./sampleStore";

export const EditorScope = createContext("local");
type Draft = { document: ManifestDocument; code: string };
const entrySource = (doc: ManifestDocument) =>
	doc.files.find((file) => file.path === doc.entrypoint)?.content ?? "";
const sourceKey = (doc: ManifestDocument) =>
	JSON.stringify([doc.folderPath, doc.entrypoint]);

export function useManifestEditor(isOpen: boolean) {
	const selected = useSelectedMap();
	const scope = useContext(EditorScope);
	const identifier = selected.id || selected.key;
	const client = useQueryClient();
	const drafts = useRef(new Map<string, Draft>());
	const [, redraw] = useState(0);
	const [status, setStatus] = useState("Ready");
	const [output, setOutput] = useState("");
	const [running, setRunning] = useState(false);
	const abort = useRef<AbortController | null>(null);
	const mounted = useRef(true);
	const selection = useRef(identifier);
	selection.current = identifier;
	const source = useQuery({
		queryKey: [...documentKey(identifier), scope],
		queryFn: () => readDocument(identifier),
		enabled: isOpen && Boolean(identifier),
		staleTime: 0,
		refetchInterval: (query) =>
			query.state.data?.manifestDocument.local ? 1500 : 15000,
	});
	const loaded = source.data?.manifestDocument;
	const savedDraft = loaded ? drafts.current.get(sourceKey(loaded)) : undefined;
	const doc =
		savedDraft && loaded
			? {
					...savedDraft.document,
					identifier: loaded.identifier,
					key: loaded.key,
				}
			: loaded;
	const code = savedDraft?.code ?? (loaded ? entrySource(loaded) : "");
	const dirty = Boolean(doc && code !== entrySource(doc));
	const hasDrafts = drafts.current.size > 0;
	const conflict = Boolean(
		dirty && loaded && doc?.revision !== loaded.revision,
	);
	const previousRevision = useRef<string | undefined>(undefined);
	useEffect(() => {
		if (
			loaded?.revision &&
			previousRevision.current &&
			previousRevision.current !== loaded.revision
		) {
			void client.invalidateQueries({ queryKey: ["GetBlocks"] });
			void client.invalidateQueries({ queryKey: ["GetScryrMaps"] });
		}
		previousRevision.current = loaded?.revision;
	}, [loaded?.revision, client]);
	const setCode = (value: string) => {
		if (!loaded) return;
		if (value === entrySource(loaded)) drafts.current.delete(sourceKey(loaded));
		else
			drafts.current.set(sourceKey(loaded), {
				document: doc ?? loaded,
				code: value,
			});
		redraw((n) => n + 1);
	};
	useEffect(() => {
		mounted.current = true;
		return () => {
			mounted.current = false;
			abort.current?.abort();
		};
	}, []);
	useEffect(() => {
		if (!hasDrafts) return;
		const protect = (event: BeforeUnloadEvent) => {
			event.preventDefault();
			event.returnValue = "";
		};
		window.addEventListener("beforeunload", protect);
		return () => window.removeEventListener("beforeunload", protect);
	}, [hasDrafts]);
	const run = async () => {
		if (!doc?.writable || running || conflict) return;
		const startedIdentifier = identifier;
		const controller = new AbortController();
		abort.current = controller;
		setRunning(true);
		setOutput("");
		setStatus("Running Python…");
		let committed = false;
		try {
			const result = await runPythonDocument(
				doc,
				code,
				(line) => setOutput((text) => `${text}${line}\n`),
				controller.signal,
			);
			if (controller.signal.aborted || !mounted.current) return;
			if (selection.current !== startedIdentifier)
				throw new Error(
					"Diagram changed during execution. Return to the draft and run again.",
				);
			setOutput((text) => text + result.stdout);
			setStatus(
				doc.local
					? "Saving file and validating…"
					: "Saving source and diagrams…",
			);
			// Cancellation stops execution; once saving starts its committed result must be reconciled.
			abort.current = null;
			const { saveManifestDocument: saved } = await saveDocument(
				doc,
				result.envelope,
			);
			committed = true;
			const latestDraft = drafts.current.get(sourceKey(doc));
			if (!latestDraft || latestDraft.code === code)
				drafts.current.delete(sourceKey(doc));
			else
				drafts.current.set(sourceKey(doc), {
					document: saved,
					code: latestDraft.code,
				});
			if (!mounted.current) return;
			if (selection.current === startedIdentifier) {
				const url = new URL(window.location.href);
				url.searchParams.set("diagram", saved.identifier);
				window.history.replaceState({}, "", url);
				mapSelectionStore.set({ id: saved.identifier, key: saved.key });
			}
			client.setQueryData([...documentKey(saved.identifier), scope], {
				manifestDocument: saved,
			});
			await client.invalidateQueries({ queryKey: ["ManifestDocument"] });
			await client.invalidateQueries({ queryKey: ["GetScryrMaps"] });
			// fetchQuery otherwise reuses the default five-minute fresh blocks cache.
			await client.invalidateQueries({
				queryKey: ["GetBlocks"],
				refetchType: "none",
			});
			const variables = { scryIdentifier: saved.identifier };
			await client.fetchQuery<GetBlocksQuery>({
				queryKey: useGetBlocksQuery.getKey(variables),
				queryFn: graphqlFetcher<GetBlocksQuery, GetBlocksQueryVariables>(
					GetBlocksDocument,
					variables,
				),
				staleTime: 0,
			});
			setStatus(
				doc.local
					? "Saved to disk · Diagram updated"
					: "Saved to cloud · Diagram updated",
			);
		} catch (error) {
			if (mounted.current) {
				setStatus(
					committed
						? "Source saved; diagram refresh failed. Reload to retry."
						: "Run failed",
				);
				setOutput(
					(text) =>
						`${text}\n${error instanceof Error ? error.message : String(error)}`,
				);
			}
		} finally {
			abort.current = null;
			if (mounted.current) setRunning(false);
		}
	};
	const reload = async () => {
		if (loaded) drafts.current.delete(sourceKey(loaded));
		await source.refetch();
		redraw((n) => n + 1);
		setStatus("Reloaded source");
	};
	return {
		doc,
		code,
		setCode,
		dirty,
		conflict,
		running,
		run,
		reload,
		cancel: () => abort.current?.abort(),
		canCancel: running && abort.current !== null,
		status: source.error
			? `Unable to load source: ${source.error.message}`
			: source.isLoading
				? "Loading source…"
				: !identifier
					? "Select a diagram to edit"
					: status,
		output,
	};
}
