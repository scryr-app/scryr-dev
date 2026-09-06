"""Runtime helpers for loading and serializing Scryr manifest files."""

from __future__ import annotations

import contextlib
import hashlib
import importlib.abc
import importlib.machinery
import importlib.util
import inspect
import io
import json
import re
import sys
from pathlib import Path
from types import CodeType, UnionType
from typing import TYPE_CHECKING, Any, Union, cast, get_args, get_origin

from scryr.manifest import Diagram, Forge, Manifest

if TYPE_CHECKING:
    from collections.abc import Iterator, Sequence
    from types import ModuleType

ManifestJsonObject = dict[str, object]
ForgeJsonObject = dict[str, object]
DiagramJsonObject = dict[str, object]
ManifestValueRecord = dict[str, str | ManifestJsonObject]
ForgeValueRecord = dict[str, str | ForgeJsonObject]
DiagramValueRecord = dict[str, str | DiagramJsonObject]
ScryrValueRecord = ManifestValueRecord | ForgeValueRecord | DiagramValueRecord
FieldTypeMetadata = dict[str, str]
ManifestTypeRecord = dict[str, str | dict[str, FieldTypeMetadata]]
ManifestSchema = dict[str, object]
ManifestPayload = ManifestSchema | list[ScryrValueRecord] | list[ManifestTypeRecord]
SCRY_SUFFIX = ".scry"
SCRYR_VARIABLE_ALIASES_NAME = "__scryr_variable_aliases__"
SCRY_TOP_LEVEL_CONSTRUCTORS = ("Diagram", "Forge", "Manifest")
SCRY_HYPHENATED_ASSIGNMENT = re.compile(
    rf"^(?P<name>[A-Za-z0-9_][A-Za-z0-9_-]*-[A-Za-z0-9_-]*)"
    rf"(?P<assignment>\s*(?::[^=]+)?=\s*)"
    rf"(?P<constructor>{'|'.join(SCRY_TOP_LEVEL_CONSTRUCTORS)})"
    rf"(?P<rest>\s*\(.*)$",
)


class ScrySourceLoader(importlib.machinery.SourceFileLoader):
    """Load Python-syntax Scryr manifest modules from `.scry` files."""

    def source_to_code(
        self,
        data: Any,  # noqa: ANN401 - matches the broad stdlib loader override.
        path: Any,  # noqa: ANN401 - matches the broad stdlib loader override.
        *,
        _optimize: int = -1,
    ) -> CodeType:
        """Compile `.scry` source after applying Scryr syntax conveniences."""
        if Path(str(path)).suffix == SCRY_SUFFIX and isinstance(data, bytes | str):
            source = importlib.util.decode_source(data) if isinstance(data, bytes) else data
            data = normalize_scry_source(source)
        return super().source_to_code(data, path, _optimize=_optimize)


def normalize_scry_source(source: str) -> str:
    """Rewrite Scryr-only top-level assignment conveniences into Python syntax."""
    aliases: dict[str, str] = {}
    lines: list[str] = []

    for line_number, line in enumerate(source.splitlines(keepends=True), start=1):
        match = SCRY_HYPHENATED_ASSIGNMENT.match(line)
        if match is None:
            lines.append(line)
            continue

        declared_name = match.group("name")
        python_name = f"scryr_scry_{line_number}_{python_identifier_from_scry_name(declared_name)}"
        aliases[python_name] = declared_name
        lines.append(
            f"{python_name}"
            f"{match.group('assignment')}"
            f"{match.group('constructor')}"
            f"{match.group('rest')}"
        )

    if aliases:
        lines.append(f"\n{SCRYR_VARIABLE_ALIASES_NAME} = {aliases!r}\n")

    return "".join(lines)


def python_identifier_from_scry_name(value: str) -> str:
    """Return a valid Python identifier fragment for a `.scry` declaration name."""
    output = []
    for character in value:
        if character == "_" or (character.isascii() and character.isalnum()):
            output.append(character)
        elif not output or output[-1] != "_":
            output.append("_")

    while output and output[-1] == "_":
        output.pop()

    return "".join(output) or "value"


class ScryPathFinder(importlib.abc.MetaPathFinder):
    """Resolve top-level Scryr manifest imports from configured source roots."""

    def __init__(self, source_roots: Sequence[Path]) -> None:
        """Create a finder that searches manifest-local roots for `.scry` files."""
        self.source_roots = tuple(source_roots)

    def find_spec(
        self,
        fullname: str,
        path: object | None = None,
        target: ModuleType | None = None,
    ) -> importlib.machinery.ModuleSpec | None:
        """Return an import spec for a `.scry` module when one exists."""
        del target
        search_paths = self._search_paths(path)
        module_parts = fullname.split(".")
        module_path = Path(*module_parts)

        for root in search_paths:
            module_file = root / module_path.with_suffix(SCRY_SUFFIX)
            if module_file.is_file():
                loader = ScrySourceLoader(fullname, str(module_file))
                return importlib.util.spec_from_file_location(fullname, module_file, loader=loader)

            package_file = root / module_path / f"__init__{SCRY_SUFFIX}"
            if package_file.is_file():
                loader = ScrySourceLoader(fullname, str(package_file))
                return importlib.util.spec_from_file_location(
                    fullname,
                    package_file,
                    loader=loader,
                    submodule_search_locations=[str(package_file.parent)],
                )

        return None

    def _search_paths(self, path: object | None) -> tuple[Path, ...]:
        if path is None:
            return self.source_roots
        if not isinstance(path, list):
            return self.source_roots
        paths = [Path(item) for item in path if isinstance(item, str)]
        paths.extend(self.source_roots)
        return tuple(dict.fromkeys(paths))


@contextlib.contextmanager
def manifest_import_context(entry_file: Path) -> Iterator[None]:
    """Temporarily enable manifest-local Python and `.scry` imports."""
    source_roots = tuple(
        dict.fromkeys(root.resolve() for root in (entry_file.parent, Path.cwd()) if root.exists())
    )
    finder = ScryPathFinder(source_roots)
    sys.meta_path.insert(0, finder)
    inserted_paths: list[str] = []

    for root in reversed(source_roots):
        root_text = str(root)
        if root_text in sys.path:
            continue
        sys.path.insert(0, root_text)
        inserted_paths.append(root_text)

    try:
        yield
    finally:
        with contextlib.suppress(ValueError):
            sys.meta_path.remove(finder)
        for root_text in inserted_paths:
            with contextlib.suppress(ValueError):
                sys.path.remove(root_text)


def _module_name_for_path(manifest_file: Path) -> str:
    stem = manifest_file.stem
    if manifest_file.suffix != SCRY_SUFFIX:
        return stem
    digest = hashlib.sha256(str(manifest_file.resolve()).encode()).hexdigest()[:12]
    return f"_scryr_scry_{stem}_{digest}"


def _spec_from_manifest_path(
    manifest_file: Path,
    module_name: str | None,
) -> importlib.machinery.ModuleSpec | None:
    name = module_name or _module_name_for_path(manifest_file)
    if manifest_file.suffix == SCRY_SUFFIX:
        loader = ScrySourceLoader(name, str(manifest_file))
        return importlib.util.spec_from_file_location(name, manifest_file, loader=loader)
    return importlib.util.spec_from_file_location(name, manifest_file)


def _type_name(value: object) -> str:
    cls = type(value)
    if cls.__module__ == "builtins":
        return cls.__qualname__
    return f"{cls.__module__}.{cls.__qualname__}"


def _collection_annotation_name(origin: object, args: tuple[object, ...]) -> str | None:
    if origin is list:
        item = _annotation_type_name(args[0]) if args else "unknown"
        return f"list[{item}]"
    if origin is tuple:
        items = ", ".join(_annotation_type_name(arg) for arg in args) if args else ""
        return f"tuple[{items}]"
    if origin is set:
        item = _annotation_type_name(args[0]) if args else "unknown"
        return f"set[{item}]"
    if origin is dict:
        key = _annotation_type_name(args[0]) if len(args) > 0 else "unknown"
        value = _annotation_type_name(args[1]) if len(args) > 1 else "unknown"
        return f"dict[{key}, {value}]"
    return None


def _annotation_type_name(annotation: object) -> str:
    origin = get_origin(annotation)
    if origin is None:
        if annotation is None or annotation is type(None):
            result = "None"
        elif isinstance(annotation, str):
            result = annotation
        elif inspect.isclass(annotation):
            result = (
                annotation.__qualname__
                if annotation.__module__ == "builtins"
                else f"{annotation.__module__}.{annotation.__qualname__}"
            )
        else:
            result = str(annotation)
        return result

    args = get_args(annotation)

    if origin in (Union, UnionType):
        variants = sorted({_annotation_type_name(arg) for arg in args})
        return " | ".join(variants)

    collection_name = _collection_annotation_name(origin, args)
    if collection_name is not None:
        return collection_name

    if inspect.isclass(origin):
        name = (
            origin.__qualname__
            if origin.__module__ == "builtins"
            else f"{origin.__module__}.{origin.__qualname__}"
        )
    else:
        name = str(origin)

    if not args:
        return name
    args_rendered = ", ".join(_annotation_type_name(arg) for arg in args)
    return f"{name}[{args_rendered}]"


def _typed_empty_collection_name(origin: object, annotation: object, fallback: str) -> str:
    if annotation is not None and get_origin(annotation) is origin:
        args = get_args(annotation)
        if args:
            if origin is dict and len(args) == 2:
                return f"dict[{_annotation_type_name(args[0])}, {_annotation_type_name(args[1])}]"
            return f"{fallback}[{_annotation_type_name(args[0])}]"
    return f"{fallback}[unknown]" if origin is not dict else "dict[unknown, unknown]"


def _value_type_name(value: object, annotation: object) -> str:
    if isinstance(value, list):
        result = (
            f"list[{', '.join(sorted({_value_type_name(item, None) for item in value}))}]"
            if value
            else _typed_empty_collection_name(list, annotation, "list")
        )
    elif isinstance(value, tuple):
        result = (
            f"tuple[{', '.join(_value_type_name(item, None) for item in value)}]"
            if value
            else "tuple[]"
        )
    elif isinstance(value, set):
        result = (
            f"set[{', '.join(sorted({_value_type_name(item, None) for item in value}))}]"
            if value
            else _typed_empty_collection_name(set, annotation, "set")
        )
    elif isinstance(value, dict):
        result = (
            "dict["
            f"{', '.join(sorted({_value_type_name(key, None) for key in value}))}, "
            f"{', '.join(sorted({_value_type_name(val, None) for val in value.values()}))}]"
            if value
            else _typed_empty_collection_name(dict, annotation, "dict")
        )
    else:
        result = _type_name(value)
    return result


def load_module_from_path(
    py_file: str | Path,
    *,
    module_name: str | None = None,
    suppress_stdout: bool = True,
) -> ModuleType:
    """Import a Python file as a module.

    Manifest files often print examples or diagnostics when run directly. Keep
    ``suppress_stdout`` enabled for library and CLI consumers that need clean
    JSON payloads.
    """
    manifest_file = Path(py_file).resolve()
    spec = _spec_from_manifest_path(manifest_file, module_name)
    if spec is None or spec.loader is None:
        msg = f"Unable to create import spec for {manifest_file}"
        raise ImportError(msg)

    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    with manifest_import_context(manifest_file):
        if suppress_stdout:
            with contextlib.redirect_stdout(io.StringIO()):
                spec.loader.exec_module(module)
        else:
            spec.loader.exec_module(module)
    return module


def load_manifest_module(py_file: str | Path) -> ModuleType:
    """Import a manifest Python or `.scry` file and suppress stdout during execution."""
    return load_module_from_path(py_file, suppress_stdout=True)


def iter_manifest_objects(module: ModuleType) -> Iterator[tuple[str, Manifest]]:
    """Yield public ``Manifest`` instances defined in a loaded module."""
    for variable, obj in inspect.getmembers(module):
        if isinstance(obj, Manifest) and not variable.startswith("_"):
            yield variable, obj


def iter_forge_objects(module: ModuleType) -> Iterator[tuple[str, Forge]]:
    """Yield public ``Forge`` instances defined in a loaded module."""
    for variable, obj in inspect.getmembers(module):
        if isinstance(obj, Forge) and not variable.startswith("_"):
            yield variable, obj


def iter_diagram_objects(module: ModuleType) -> Iterator[tuple[str, Diagram]]:
    """Yield public ``Diagram`` instances defined in a loaded module."""
    for variable, obj in inspect.getmembers(module):
        if isinstance(obj, Diagram) and not variable.startswith("_"):
            yield variable, obj


def scryr_variable_name(module: ModuleType, python_variable: str) -> str:
    """Return the declared Scryr variable name for a module binding."""
    aliases = getattr(module, SCRYR_VARIABLE_ALIASES_NAME, {})
    if isinstance(aliases, dict) and isinstance(aliases.get(python_variable), str):
        return cast("str", aliases[python_variable])
    return python_variable


def emit_manifest_values(module: ModuleType) -> list[ManifestValueRecord]:
    """Serialize public manifest instances from a loaded module."""
    return [
        {
            "kind": "manifest",
            "variable_name": scryr_variable_name(module, variable),
            "manifest": cast("ManifestJsonObject", json.loads(obj.model_dump_json())),
        }
        for variable, obj in iter_manifest_objects(module)
    ]


def emit_forge_values(module: ModuleType) -> list[ForgeValueRecord]:
    """Serialize public forge instances from a loaded module."""
    return [
        {
            "kind": "forge",
            "variable_name": scryr_variable_name(module, variable),
            "forge": cast("ForgeJsonObject", json.loads(obj.model_dump_json())),
        }
        for variable, obj in iter_forge_objects(module)
    ]


def emit_diagram_values(module: ModuleType) -> list[DiagramValueRecord]:
    """Serialize public diagram instances from a loaded module."""
    manifest_pool = [obj for _, obj in iter_manifest_objects(module)]
    return [
        {
            "kind": "diagram",
            "variable_name": scryr_variable_name(module, variable),
            "diagram": cast("DiagramJsonObject", obj.to_dict(manifest_pool=manifest_pool)),
        }
        for variable, obj in iter_diagram_objects(module)
    ]


def emit_scryr_values(module: ModuleType) -> list[ScryrValueRecord]:
    """Serialize all public top-level Scryr constructs from a loaded module."""
    return [*emit_manifest_values(module), *emit_forge_values(module), *emit_diagram_values(module)]


def emit_manifest_types(module: ModuleType) -> list[ManifestTypeRecord]:
    """Describe declared and runtime field types for public top-level constructs."""
    block_types: list[ManifestTypeRecord] = []
    for kind, variable, obj in (
        ("manifest", variable, obj) for variable, obj in iter_manifest_objects(module)
    ):
        fields = obj.__class__.model_fields
        field_types = {
            field_name: {
                "declared": _annotation_type_name(fields[field_name].annotation),
                "runtime": _value_type_name(
                    getattr(obj, field_name),
                    fields[field_name].annotation,
                ),
            }
            for field_name in fields
        }
        block_types.append(
            {
                "kind": kind,
                "variable": scryr_variable_name(module, variable),
                "manifest_name": obj.name,
                "field_types": field_types,
            }
        )
    forge_objects = (("forge", variable, obj) for variable, obj in iter_forge_objects(module))
    for kind, variable, obj in forge_objects:
        fields = obj.__class__.model_fields
        field_types = {
            field_name: {
                "declared": _annotation_type_name(fields[field_name].annotation),
                "runtime": _value_type_name(
                    getattr(obj, field_name),
                    fields[field_name].annotation,
                ),
            }
            for field_name in fields
        }
        block_types.append(
            {
                "kind": kind,
                "variable": scryr_variable_name(module, variable),
                "forge_name": obj.name,
                "field_types": field_types,
            }
        )
    diagram_objects = (("diagram", variable, obj) for variable, obj in iter_diagram_objects(module))
    for kind, variable, obj in diagram_objects:
        fields = obj.__class__.model_fields
        field_types = {
            field_name: {
                "declared": _annotation_type_name(fields[field_name].annotation),
                "runtime": _value_type_name(
                    getattr(obj, field_name),
                    fields[field_name].annotation,
                ),
            }
            for field_name in fields
        }
        block_types.append(
            {
                "kind": kind,
                "variable": scryr_variable_name(module, variable),
                "diagram_name": obj.name,
                "field_types": field_types,
            }
        )
    return block_types


def emit_manifest_schema() -> ManifestSchema:
    """Return the serialized top-level Scryr model JSON schemas."""
    Forge.model_rebuild()
    Manifest.model_rebuild()
    Diagram.model_rebuild()
    return {
        "title": "ScryrManifestSchema",
        "models": {
            "Manifest": cast(
                "ManifestJsonObject",
                Manifest.model_json_schema(mode="serialization"),
            ),
            "Forge": cast("ForgeJsonObject", Forge.model_json_schema(mode="serialization")),
            "Diagram": cast(
                "DiagramJsonObject",
                Diagram.model_json_schema(mode="serialization"),
            ),
        },
    }


def run_manifest_file(manifest_file: str | Path, mode: str = "--json") -> ManifestPayload:
    """Execute one manifest file and return the requested serialized payload."""
    if mode == "--schema":
        return emit_manifest_schema()

    module = load_manifest_module(Path(manifest_file).resolve())
    if mode == "--types":
        return emit_manifest_types(module)
    return emit_scryr_values(module)
