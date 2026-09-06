import { browserSdkFiles } from "@/pyodide/sdkSources";

const PYODIDE_VERSION = "v314.0.0";
const PYODIDE_CDN = `https://cdn.jsdelivr.net/pyodide/${PYODIDE_VERSION}/full/`;
const SCRIPT_ID = "scryr-pyodide-script";
const RUNTIME_ROOT = "/scryr-runtime";

type PyProxyLike = {
	destroy?: () => void;
	toJs?: (options?: unknown) => unknown;
};

export type PyodideInstance = {
	FS: {
		analyzePath: (path: string) => { exists: boolean };
		mkdirTree: (path: string) => void;
		writeFile: (path: string, data: string) => void;
	};
	globals: {
		get: (key: string) => PyProxyLike;
	};
	loadPackage: (names: string | string[]) => Promise<void>;
	runPython: (code: string) => unknown;
	runPythonAsync: (code: string) => Promise<unknown>;
};

declare global {
	interface Window {
		loadPyodide?: (options: {
			indexURL: string;
			stderr?: (message: string) => void;
			stdout?: (message: string) => void;
		}) => Promise<PyodideInstance>;
	}
}

let pyodidePromise: Promise<PyodideInstance> | null = null;

function ensurePyodideScript(): Promise<void> {
	if (window.loadPyodide) {
		return Promise.resolve();
	}

	const existingScript = document.getElementById(SCRIPT_ID);
	if (existingScript) {
		return new Promise((resolve, reject) => {
			existingScript.addEventListener("load", () => resolve(), { once: true });
			existingScript.addEventListener(
				"error",
				() => reject(new Error("Failed to load Pyodide script.")),
				{ once: true },
			);
		});
	}

	return new Promise((resolve, reject) => {
		const script = document.createElement("script");
		script.id = SCRIPT_ID;
		script.src = `${PYODIDE_CDN}pyodide.js`;
		script.async = true;
		script.onload = () => resolve();
		script.onerror = () => reject(new Error("Failed to load Pyodide script."));
		document.head.appendChild(script);
	});
}

function ensureDirectory(pyodide: PyodideInstance, path: string) {
	if (!pyodide.FS.analyzePath(path).exists) {
		pyodide.FS.mkdirTree(path);
	}
}

function installBrowserSdk(pyodide: PyodideInstance) {
	ensureDirectory(pyodide, RUNTIME_ROOT);

	for (const [relativePath, source] of Object.entries(browserSdkFiles)) {
		const segments = relativePath.split("/");
		const directoryPath = `${RUNTIME_ROOT}/${segments.slice(0, -1).join("/")}`;
		ensureDirectory(pyodide, directoryPath);
		pyodide.FS.writeFile(`${RUNTIME_ROOT}/${relativePath}`, source);
	}

	pyodide.runPython(`
import sys
runtime_root = "${RUNTIME_ROOT}"
if runtime_root not in sys.path:
    sys.path.insert(0, runtime_root)
`);
}

export async function getPyodide(onOutput?: (line: string) => void) {
	if (!pyodidePromise) {
		pyodidePromise = (async () => {
			await ensurePyodideScript();

			const pyodide = await window.loadPyodide?.({
				indexURL: PYODIDE_CDN,
				stdout: (message) => onOutput?.(message),
				stderr: (message) => onOutput?.(message),
			});

			if (!pyodide) {
				throw new Error("Pyodide failed to initialize.");
			}

			await pyodide.loadPackage("micropip");
			installBrowserSdk(pyodide);
			return pyodide;
		})();
	}

	return pyodidePromise;
}

export type PythonRunResult = {
	manifests: PythonManifestResult[];
	diagrams: PythonDiagramResult[];
	schema: Record<string, unknown> | null;
	stdout: string;
};

export type PythonManifestResult = {
	name: string;
	manifest: Record<string, unknown>;
};

export type PythonDiagramResult = {
	name: string;
	diagram: Record<string, unknown>;
};

export async function runPythonSample(
	code: string,
	onOutput?: (line: string) => void,
) {
	const pyodide = await getPyodide(onOutput);
	const escaped = JSON.stringify(code);

	await pyodide.runPythonAsync(`
import io
import json
from contextlib import redirect_stderr, redirect_stdout

from scryr.manifest import Diagram, Manifest

_scryr_stdout = io.StringIO()
_scryr_stderr = io.StringIO()
_scryr_globals = {}

with redirect_stdout(_scryr_stdout), redirect_stderr(_scryr_stderr):
    exec(${escaped}, _scryr_globals)

_scryr_manifests = []
_scryr_diagrams = []
for _name, _value in _scryr_globals.items():
    if isinstance(_value, Manifest):
        _scryr_manifests.append({
            "name": _name,
            "manifest": _value.to_dict(),
        })
    if isinstance(_value, Diagram):
        _scryr_diagrams.append({
            "name": _name,
            "diagram": _value.to_dict(),
        })

_scryr_schema_factory = getattr(Manifest, "model_json_schema", None)
_scryr_schema = (
    _scryr_schema_factory(mode="serialization")
    if callable(_scryr_schema_factory)
    else None
)

_scryr_result = json.dumps({
    "schema": _scryr_schema,
    "stdout": _scryr_stdout.getvalue() + _scryr_stderr.getvalue(),
    "manifests": _scryr_manifests,
    "diagrams": _scryr_diagrams,
}, indent=2)
`);

	const resultProxy = pyodide.globals.get("_scryr_result");
	const resultText = String(
		resultProxy.toJs ? resultProxy.toJs() : resultProxy,
	);
	resultProxy.destroy?.();
	return JSON.parse(resultText) as PythonRunResult;
}
