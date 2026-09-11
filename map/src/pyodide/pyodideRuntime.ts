import type {
	ManifestDocument,
	ManifestEnvelope,
} from "@/graphql/manifestDocument";

export type PythonRunResult = { envelope: ManifestEnvelope; stdout: string };

// Each run uses a fresh worker: imports and globals cannot leak between documents.
export function runPythonDocument(
	document: ManifestDocument,
	code: string,
	onOutput?: (line: string) => void,
	signal?: AbortSignal,
): Promise<PythonRunResult> {
	return new Promise((resolve, reject) => {
		const worker = new Worker(new URL("./pyodideWorker.ts", import.meta.url), {
			type: "module",
		});
		const cleanup = () => {
			clearTimeout(timeout);
			signal?.removeEventListener("abort", cancel);
			worker.terminate();
		};
		const cancel = () => {
			cleanup();
			reject(new Error("Execution cancelled. Source was not saved."));
		};
		const timeout = setTimeout(() => {
			cleanup();
			reject(new Error("Python execution timed out. Source was not saved."));
		}, 120_000);
		worker.onmessage = ({ data }) => {
			if (data.output !== undefined) {
				onOutput?.(data.output);
				return;
			}
			cleanup();
			if (data.error) reject(new Error(data.error));
			else resolve(data.result as PythonRunResult);
		};
		worker.onerror = (event) => {
			cleanup();
			reject(new Error(event.message || "Python worker failed"));
		};
		signal?.addEventListener("abort", cancel, { once: true });
		if (signal?.aborted) {
			cancel();
			return;
		}
		worker.postMessage({
			entrypoint: document.entrypoint,
			files: document.files.map((file) =>
				file.path === document.entrypoint ? { ...file, content: code } : file,
			),
		});
	});
}
