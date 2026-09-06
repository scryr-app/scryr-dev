"""Command-line entrypoint and compatibility imports for Scryr manifests."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from scryr.runtime import (
    ManifestPayload,
    emit_diagram_values,
    emit_forge_values,
    emit_manifest_schema,
    emit_manifest_types,
    emit_manifest_values,
    emit_scryr_values,
    iter_diagram_objects,
    iter_forge_objects,
    iter_manifest_objects,
    load_manifest_module,
    normalize_scry_source,
    run_manifest_file,
)

ManifestCliPayload = ManifestPayload
__all__ = [
    "ManifestCliPayload",
    "emit_diagram_values",
    "emit_forge_values",
    "emit_manifest_schema",
    "emit_manifest_types",
    "emit_manifest_values",
    "emit_scryr_values",
    "iter_diagram_objects",
    "iter_forge_objects",
    "iter_manifest_objects",
    "load_manifest_module",
    "main",
    "normalize_scry_source",
    "run_manifest_cli",
]


def run_manifest_cli(manifest_file: Path, mode: str) -> ManifestCliPayload:
    """Execute the requested adapter mode for one manifest file."""
    return run_manifest_file(manifest_file, mode)


def main(argv: list[str] | None = None) -> int:
    """Command-line entrypoint used by the Rust CLI via `python -m scryr.cli`."""
    arguments = sys.argv[1:] if argv is None else argv
    if not arguments:
        sys.stderr.write("Usage: scryr <manifest.py> [--json|--types|--schema]\n")
        return 2
    manifest_file = Path(arguments[0]).resolve()
    mode = arguments[1] if len(arguments) > 1 else "--json"
    sys.stdout.write(f"{json.dumps(run_manifest_cli(manifest_file, mode), indent=2)}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
