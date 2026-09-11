"""Execute the real SDK in the worker's isolated filesystem."""
import ast
import contextlib
import io
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, "/sdk")
os.chdir("/project")

from scryr.runtime import (  # noqa: E402
    emit_scryr_values,
    load_module_from_path,
    normalize_scry_source,
    scryr_variable_name,
)

document = json.loads(_scryr_document)
entrypoint = Path(document["entrypoint"])
source = entrypoint.read_text()
tree = ast.parse(normalize_scry_source(source))
lines = {}
for node in tree.body:
    if isinstance(node, ast.Assign):
        for target in node.targets:
            if isinstance(target, ast.Name):
                lines[target.id] = node.lineno
    elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
        lines[node.target.id] = node.lineno
    elif isinstance(node, (ast.Import, ast.ImportFrom)):
        for alias in node.names:
            lines[alias.asname or alias.name] = node.lineno

output = io.StringIO()
with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
    module = load_module_from_path(entrypoint, suppress_stdout=False)
    values = emit_scryr_values(module)

lines = {scryr_variable_name(module, name): line for name, line in lines.items()}
envelope = {"files": document["files"], "manifests": [], "forges": [], "diagrams": []}
for record in values:
    kind = record["kind"]
    value = record[kind]
    variable = record["variable_name"]
    if variable not in lines:
        raise ValueError(f"Cannot find source location for {variable}")
    value["variable_name"] = variable
    value["line_number"] = lines[variable]
    envelope[{"manifest": "manifests", "forge": "forges", "diagram": "diagrams"}[kind]].append(value)

json.dumps({"envelope": envelope, "stdout": output.getvalue()})

