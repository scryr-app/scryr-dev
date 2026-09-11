import runner from "./runDocument.py?raw";
import { browserSdkFiles } from "./sdkSources";

const PYODIDE_CDN = "https://cdn.jsdelivr.net/pyodide/v314.0.0/full/";
type PythonRuntime = {
	FS: {
		mkdirTree(path: string): void;
		writeFile(path: string, value: string): void;
	};
	loadPackage(names: string[]): Promise<void>;
	runPythonAsync(code: string): Promise<unknown>;
	globals: { set(key: string, value: string): void };
};
const scope = self as unknown as {
	onmessage: (
		event: MessageEvent<{
			entrypoint: string;
			files: { path: string; content: string }[];
		}>,
	) => void;
	postMessage(value: unknown): void;
};
function safePath(path: string) {
	if (
		path.startsWith("/") ||
		path.includes("\\") ||
		path.split("/").some((part) => !part || part === "." || part === "..")
	) {
		throw new Error("Invalid source path");
	}
	return path;
}
scope.onmessage = async ({ data }) => {
	try {
		scope.postMessage({ output: "Loading Python and the Scryr SDK…" });
		const { loadPyodide } = await import(
			/* @vite-ignore */ `${PYODIDE_CDN}pyodide.mjs`
		);
		const output = (message: string) => scope.postMessage({ output: message });
		const python: PythonRuntime = await loadPyodide({
			indexURL: PYODIDE_CDN,
			stdout: output,
			stderr: output,
		});
		await python.loadPackage(["pydantic"]);
		for (const [path, source] of Object.entries(browserSdkFiles)) {
			python.FS.mkdirTree(`/sdk/${path.slice(0, path.lastIndexOf("/"))}`);
			python.FS.writeFile(`/sdk/${path}`, source);
		}
		for (const file of data.files) {
			const path = safePath(file.path);
			python.FS.mkdirTree(
				`/project/${path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : ""}`,
			);
			python.FS.writeFile(`/project/${path}`, file.content);
		}
		safePath(data.entrypoint);
		python.globals.set("_scryr_document", JSON.stringify(data));
		const text = await python.runPythonAsync(runner);
		scope.postMessage({ result: JSON.parse(String(text)) });
	} catch (error) {
		scope.postMessage({
			error: error instanceof Error ? error.message : String(error),
		});
	}
};
