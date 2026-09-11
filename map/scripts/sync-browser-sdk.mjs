import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join, relative, sep } from "node:path";

const root = fileURLToPath(new URL("../../manifest/scryr/src/scryr/", import.meta.url));
const output = new URL("../src/pyodide/sdkSources.generated.json", import.meta.url);
if (existsSync(root)) {
  const files = {};
  function collect(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const path = join(directory, entry.name);
      if (entry.isDirectory() && entry.name !== "__pycache__") collect(path);
      else if (entry.isFile() && entry.name.endsWith(".py")) {
        files[`scryr/${relative(root, path).split(sep).join("/")}`] = readFileSync(path, "utf8");
      }
    }
  }
  collect(root);
  if (!files["scryr/runtime.py"]) throw new Error("Python SDK runtime is missing");
  writeFileSync(output, `${JSON.stringify(files, null, "\t")}\n`);
} else {
  const files = JSON.parse(readFileSync(output, "utf8"));
  if (!files["scryr/runtime.py"]) throw new Error("Bundled Python SDK runtime is missing");
}
