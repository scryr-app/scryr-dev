"""Shared manifest discovery helpers for the library and CLI."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Any

from pydantic import BaseModel

from .manifest import Manifest
from .runtime import SCRY_SUFFIX, load_module_from_path

if TYPE_CHECKING:
    import io
    from collections.abc import Iterator


def _iter_manifest_files(search_dir: Path) -> Iterator[Path]:
    """Yield importable Python and `.scry` files from *search_dir*."""
    manifest_files = [
        path
        for path in search_dir.rglob("*")
        if path.is_file() and path.suffix in {".py", SCRY_SUFFIX} and not path.name.startswith("_")
    ]
    yield from sorted(manifest_files)


def discover_pydantic_instances(
    search_path: Path | None = None,
    *,
    strict: bool = False,
    stderr: io.TextIOBase | None = None,
) -> list[dict[str, Any]]:
    """Discover Pydantic model instances in *search_path*."""
    if search_path is None:
        samples_dir = Path(__file__).resolve().parents[3] / "samples"
    else:
        samples_dir = Path(search_path)

    if not samples_dir.exists():
        msg = f"Path does not exist: {samples_dir}"
        raise ValueError(msg)
    if not samples_dir.is_dir():
        msg = f"Path is not a directory: {samples_dir}"
        raise ValueError(msg)

    results: list[dict[str, Any]] = []
    err_stream = stderr if stderr is not None else sys.stderr

    for py_file in _iter_manifest_files(samples_dir):
        relative_path = py_file.relative_to(samples_dir)

        try:
            module = load_module_from_path(py_file, suppress_stdout=False)
        except Exception as exc:
            if strict:
                raise
            print(f"Warning: failed to load {py_file}: {exc}", file=err_stream)
            continue

        for name, obj in vars(module).items():
            if not isinstance(obj, BaseModel) or name.startswith("_"):
                continue
            try:
                json_data = json.loads(obj.model_dump_json())
            except Exception as exc:
                if strict:
                    raise
                print(
                    f"Warning: failed to serialize {relative_path}:{name}: {exc}",
                    file=err_stream,
                )
                continue

            results.append(
                {
                    "name": name,
                    "file": str(relative_path),
                    "data": json_data,
                }
            )

    return results


def collect_manifest_blocks(
    samples_dir: Path,
    *,
    strict: bool = False,
    stderr: io.TextIOBase | None = None,
) -> list[dict[str, Any]]:
    """Load every sample file in *samples_dir* and return Manifest instances as dicts."""
    blocks: list[dict[str, Any]] = []
    err_stream = stderr if stderr is not None else sys.stderr

    for py_file in _iter_manifest_files(samples_dir):
        try:
            module = load_module_from_path(py_file, suppress_stdout=True)
        except Exception as exc:
            if strict:
                raise
            print(f"Warning: failed to load {py_file}: {exc}", file=err_stream)
            continue

        for name, obj in vars(module).items():
            if not isinstance(obj, Manifest) or name.startswith("_"):
                continue
            try:
                blocks.append(json.loads(obj.model_dump_json()))
            except Exception as exc:
                if strict:
                    raise
                print(
                    f"Warning: failed to serialize {py_file.name}:{name}: {exc}",
                    file=err_stream,
                )

    return blocks
