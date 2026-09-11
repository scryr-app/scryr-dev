"""Source-only checks for manifest entrypoints and their local imports."""

# CLI subprocesses are argv-only, run pinned tools from the managed environment,
# and intentionally print diagnostics.
# ruff: noqa: T201, S603, S607
from __future__ import annotations

import ast
import io
import subprocess
import sys
import tempfile
import tokenize
from pathlib import Path

from scryr.runtime import normalize_scry_source


def sources(entry: Path) -> list[Path]:
    """Collect reachable local Python and Scryr modules without executing them."""
    root = entry.parent
    pending = [entry]
    seen: set[Path] = set()
    while pending:
        path = pending.pop().resolve()
        if path in seen:
            continue
        seen.add(path)
        tree = ast.parse(normalize_scry_source(path.read_text()), filename=str(path))
        for node in ast.walk(tree):
            modules: list[str] = []
            base = root
            if isinstance(node, ast.Import):
                modules = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                if node.level:
                    base = path.parent
                    for _ in range(node.level - 1):
                        base = base.parent
                module = node.module or ""
                modules = [module, *[f"{module}.{a.name}".strip(".") for a in node.names]]
            for module in modules:
                target = base.joinpath(*module.split("."))
                for candidate in (
                    target.with_suffix(".py"),
                    target.with_suffix(".scry"),
                    target / "__init__.py",
                    target / "__init__.scry",
                ):
                    if candidate.is_file() and candidate.resolve().is_relative_to(root):
                        pending.append(candidate)
                        # Include package initialization for dotted imports.
                        for parent in candidate.parents:
                            if parent == root:
                                break
                            for suffix in (".py", ".scry"):
                                init = parent / f"__init__{suffix}"
                                if init.is_file():
                                    pending.append(init)
                        break
    return sorted(seen)


def _normalized(source: str) -> tuple[str, dict[str, str]]:
    """Normalize declaration aliases, retaining names for source round trips."""
    normalized = normalize_scry_source(source)
    if normalized == source:
        return source, {}
    tree = ast.parse(normalized)
    last = tree.body[-1]
    if isinstance(last, ast.Assign) and isinstance(last.value, ast.Dict):
        aliases = ast.literal_eval(last.value)
        lines = normalized.splitlines(keepends=True)
        return "".join(lines[: last.lineno - 1]).rstrip() + "\n", aliases
    return normalized, {}


def _restore_aliases(source: str, aliases: dict[str, str]) -> str:
    """Restore declaration identifiers without changing string literals or comments."""
    offsets = [0]
    for line in source.splitlines(keepends=True):
        offsets.append(offsets[-1] + len(line))
    edits = []
    for token in tokenize.generate_tokens(io.StringIO(source).readline):
        if token.type == tokenize.NAME and token.string in aliases:
            start = offsets[token.start[0] - 1] + token.start[1]
            end = offsets[token.end[0] - 1] + token.end[1]
            edits.append((start, end, aliases[token.string]))
    for start, end, replacement in reversed(edits):
        source = source[:start] + replacement + source[end:]
    return source


def _format_check_failed(
    result: subprocess.CompletedProcess[str], normalized: str, path: Path
) -> bool:
    """Report why a source would fail the nonmutating formatting stage."""
    if result.returncode != 0:
        if result.stdout:
            print(result.stdout, end="")
        if result.stderr:
            print(result.stderr, end="", file=sys.stderr)
        return True
    if result.stdout != normalized:
        print(f"Would reformat {path}")
        return True
    return False


def run(entry: Path, mode: str) -> int:
    """Run pinned tools; type checking uses temporary Python import mirrors."""
    files = sources(entry)
    if mode == "--sources":
        import json

        print(json.dumps([str(p) for p in files]))
        return 0
    if mode == "--typecheck":
        with tempfile.TemporaryDirectory(prefix="scryr-types-") as temporary:
            root = Path(temporary)
            mirrors = []
            names = {}
            for source in files:
                mirror = (root / source.relative_to(entry.parent)).with_suffix(".py")
                mirror.parent.mkdir(parents=True, exist_ok=True)
                mirror.write_text(normalize_scry_source(source.read_text()))
                mirrors.append(str(mirror))
                names[str(mirror)] = str(source)
            result = subprocess.run(
                [
                    "ty",
                    "check",
                    "--python",
                    sys.executable,
                    "--extra-search-path",
                    str(root),
                    "--project",
                    str(entry.parent),
                    *mirrors,
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            output = result.stdout + result.stderr
            for mirror, source in names.items():
                output = output.replace(mirror, source)
            print(output, end="")
            return result.returncode
    failed = False
    for path in files:
        original = path.read_text()
        normalized, aliases = _normalized(original)
        formatting = mode in ("--format", "--format-check")
        args = [
            "ruff",
            "format" if formatting else "check",
            "--extension",
            "scry:python",
            "--stdin-filename",
            str(path),
        ]
        if not formatting:
            # Imported public objects are manifest exports, not unused imports.
            args.extend(["--extend-per-file-ignores", "*:F401"])
        if mode == "--lint-fix":
            args.extend(["--fix", "--no-unsafe-fixes"])
        result = subprocess.run(
            [*args, "-"], input=normalized, capture_output=True, text=True, check=False
        )
        if mode == "--format-check":
            failed |= _format_check_failed(result, normalized, path)
            continue
        writing = mode in ("--format", "--lint-fix")
        if writing and result.stdout:
            updated = _restore_aliases(result.stdout, aliases)
            if updated != original:
                path.write_text(updated)
                print(f"Updated {path}")
        elif result.stdout:
            print(result.stdout, end="")
        if result.stderr:
            print(result.stderr, end="", file=sys.stderr)
        failed |= result.returncode != 0
    return int(failed)
