"""Tests for the Python-side Scryr CLI adapter helpers."""

import json
from pathlib import Path
from typing import cast

from scryr.cli import (
    emit_diagram_values,
    emit_forge_values,
    emit_manifest_schema,
    load_manifest_module,
    normalize_scry_source,
    run_manifest_cli,
)


def test_run_manifest_cli_types_returns_sample_metadata() -> None:
    """The adapter library emits type metadata for the sample Open SaaS manifest."""
    manifest_dir = Path(__file__).parent.parent
    payload = run_manifest_cli(
        manifest_dir / "samples" / "open_saas" / "index.scry",
        "--types",
    )

    assert isinstance(payload, list)
    assert any(block["variable"] == "postgres_block" for block in payload)
    assert any(block["manifest_name"] == "PostgreSQL" for block in payload)


def test_load_manifest_module_suppresses_sample_stdout(capsys) -> None:
    """Loading a manifest module through the adapter does not leak print output."""
    manifest_dir = Path(__file__).parent.parent
    module = load_manifest_module(
        manifest_dir / "samples" / "open_saas" / "index.scry",
    )

    captured = capsys.readouterr()
    assert captured.out == ""
    assert module.__name__.startswith("_scryr_scry_index_")


def test_emit_manifest_schema_matches_manifest_shape() -> None:
    """The adapter library exposes the serialized Manifest schema."""
    schema = emit_manifest_schema()

    assert isinstance(schema, dict)
    assert "models" in schema
    models = cast("dict[str, object]", schema["models"])
    assert "Manifest" in models
    assert "Forge" in models
    assert "Diagram" in models


def test_run_manifest_cli_json_serializes_manifest_values() -> None:
    """JSON mode returns serializable manifest values for the sample manifest."""
    manifest_dir = Path(__file__).parent.parent
    payload = run_manifest_cli(
        manifest_dir / "samples" / "open_saas" / "index.scry",
        "--json",
    )

    assert isinstance(payload, list)
    assert json.loads(json.dumps(payload)) == payload


def test_sample_query_diagrams_resolve_selected_manifests() -> None:
    """Sample query-backed diagrams serialize with their selected manifest blocks."""
    manifest_dir = Path(__file__).parent.parent
    payload = run_manifest_cli(
        manifest_dir / "samples" / "open_saas" / "index.scry",
        "--json",
    )

    assert isinstance(payload, list)
    assert _diagram_manifest_names(payload, "OpenSaaS External Integrations") == [
        "Analytics",
        "Email",
        "Stripe",
    ]


def test_run_manifest_cli_json_serializes_forge_values(tmp_path: Path) -> None:
    """JSON mode returns public top-level forge and diagram values alongside manifests."""
    manifest_file = tmp_path / "main.scry"
    manifest_file.write_text(
        "from scryr import Diagram, Forge, Manifest\n"
        'node = Forge(name="Node", tools={"node": "22"})\n'
        'web = Manifest(name="Web", forges=["Node"])\n'
        'main_diagram = Diagram(name="Main", manifests=[web])\n',
    )

    payload = run_manifest_cli(manifest_file, "--json")

    assert isinstance(payload, list)
    assert any(_is_named_forge(record, "Node") for record in payload)
    assert any(_is_named_manifest(record, "Web") for record in payload)
    assert any(_is_named_diagram(record, "Main") for record in payload)


def test_emit_forge_values_serializes_public_forges(tmp_path: Path) -> None:
    """The adapter exposes a focused helper for public forge instances."""
    manifest_file = tmp_path / "main.scry"
    manifest_file.write_text(
        'from scryr import Forge\nnode = Forge(name="Node", tools={"node": "22"})\n',
    )
    module = load_manifest_module(manifest_file)

    payload = emit_forge_values(module)

    assert payload[0]["kind"] == "forge"
    assert payload[0]["variable_name"] == "node"


def test_emit_diagram_values_serializes_public_diagrams(tmp_path: Path) -> None:
    """The adapter exposes a focused helper for public diagram instances."""
    manifest_file = tmp_path / "main.scry"
    manifest_file.write_text(
        "from scryr import Diagram, Manifest\n"
        'web = Manifest(name="Web")\n'
        'main = Diagram(name="Main", manifests=[web])\n',
    )
    module = load_manifest_module(manifest_file)

    payload = emit_diagram_values(module)

    assert payload[0]["kind"] == "diagram"
    assert payload[0]["variable_name"] == "main"


def test_emit_diagram_values_resolves_query_backed_diagrams(tmp_path: Path) -> None:
    """A Diagram query filters public manifests from the same module during emission."""
    manifest_file = tmp_path / "main.scry"
    manifest_file.write_text(
        "from scryr import Diagram, Manifest, ProgrammingLanguage\n"
        'api = Manifest(name="API", language=ProgrammingLanguage.python, tags=["backend"])\n'
        'web = Manifest(name="Web", language=ProgrammingLanguage.typescript, tags=["frontend"])\n'
        'main = Diagram(name="Backend", query=Diagram.Query(language="python", tags="backend"))\n',
    )
    module = load_manifest_module(manifest_file)

    payload = emit_diagram_values(module)

    diagram = cast("dict[str, object]", payload[0]["diagram"])
    manifests = cast("list[dict[str, object]]", diagram["manifests"])
    assert [manifest["name"] for manifest in manifests] == ["API"]


def test_scry_loader_supports_hyphenated_diagram_variable_names(tmp_path: Path) -> None:
    """A `.scry` Diagram declaration can use a slug as its Scryr identifier."""
    manifest_file = tmp_path / "main.scry"
    manifest_file.write_text(
        "from scryr import Diagram, Manifest\n"
        'web = Manifest(name="Web")\n'
        'ex-northwind-commerce-diagram = Diagram(name="Northwind", manifests=[web])\n',
    )

    payload = run_manifest_cli(manifest_file, "--json")

    assert isinstance(payload, list)
    diagram = next(record for record in payload if _is_named_diagram(record, "Northwind"))
    assert diagram["variable_name"] == "ex-northwind-commerce-diagram"


def test_scry_types_preserve_hyphenated_diagram_variable_names(tmp_path: Path) -> None:
    """Type metadata uses the declared Scryr variable name for slugged diagrams."""
    manifest_file = tmp_path / "main.scry"
    manifest_file.write_text(
        "from scryr import Diagram, Manifest\n"
        'web = Manifest(name="Web")\n'
        'ex-northwind-commerce-diagram = Diagram(name="Northwind", manifests=[web])\n',
    )

    payload = run_manifest_cli(manifest_file, "--types")

    assert isinstance(payload, list)
    assert any(
        record["kind"] == "diagram" and record["variable"] == "ex-northwind-commerce-diagram"
        for record in payload
    )


def test_scry_normalization_keeps_existing_line_numbers() -> None:
    """Slugged Scryr declarations are rewritten without shifting source lines."""
    source = (
        "from scryr import Diagram, Manifest\n"
        'web = Manifest(name="Web")\n'
        'ex-northwind-commerce-diagram = Diagram(name="Northwind", manifests=[web])\n'
    )

    normalized = normalize_scry_source(source)

    assert normalized.splitlines()[2].startswith("scryr_scry_3_ex_northwind_commerce_diagram")
    assert "__scryr_variable_aliases__" in normalized


def test_scry_manifest_imports_python_and_scry_modules(tmp_path: Path) -> None:
    """A `.scry` entrypoint can import sibling Python modules and `.scry` modules."""
    (tmp_path / "names.py").write_text('PYTHON_NAME = "Python helper"\n')
    (tmp_path / "shared.scry").write_text(
        'from scryr import Manifest\nshared_manifest = Manifest(name="Shared Scry")\n',
    )
    manifest_file = tmp_path / "main.scry"
    manifest_file.write_text(
        "from scryr import Manifest\n"
        "from names import PYTHON_NAME\n"
        "from shared import shared_manifest\n"
        'api = Manifest(name=f"{PYTHON_NAME} + {shared_manifest.name}")\n',
    )

    payload = run_manifest_cli(manifest_file, "--json")

    assert isinstance(payload, list)
    assert any(_is_imported_scry_manifest(block) for block in payload)


def _is_imported_scry_manifest(block: object) -> bool:
    if not isinstance(block, dict):
        return False

    record = cast("dict[str, object]", block)
    if record.get("variable_name") != "api":
        return False

    manifest = record.get("manifest")
    if not isinstance(manifest, dict):
        return False

    manifest_record = cast("dict[str, object]", manifest)
    return manifest_record.get("name") == "Python helper + Shared Scry"


def _is_named_forge(record: object, name: str) -> bool:
    if not isinstance(record, dict):
        return False
    record = cast("dict[str, object]", record)
    forge = record.get("forge")
    if not isinstance(forge, dict):
        return False
    forge = cast("dict[str, object]", forge)
    return forge.get("name") == name


def _is_named_manifest(record: object, name: str) -> bool:
    if not isinstance(record, dict):
        return False
    record = cast("dict[str, object]", record)
    manifest = record.get("manifest")
    if not isinstance(manifest, dict):
        return False
    manifest = cast("dict[str, object]", manifest)
    return manifest.get("name") == name


def _is_named_diagram(record: object, name: str) -> bool:
    if not isinstance(record, dict):
        return False
    record = cast("dict[str, object]", record)
    diagram = record.get("diagram")
    if not isinstance(diagram, dict):
        return False
    diagram = cast("dict[str, object]", diagram)
    return diagram.get("name") == name


def _diagram_manifest_names(records: object, diagram_name: str) -> list[str]:
    if not isinstance(records, list):
        return []

    for record in records:
        if not isinstance(record, dict):
            continue
        typed_record = cast("dict[str, object]", record)
        diagram = typed_record.get("diagram")
        if not isinstance(diagram, dict):
            continue
        typed_diagram = cast("dict[str, object]", diagram)
        if typed_diagram.get("name") != diagram_name:
            continue
        manifests = typed_diagram.get("manifests")
        if not isinstance(manifests, list):
            return []
        manifest_names: list[str] = []
        for manifest in manifests:
            if not isinstance(manifest, dict):
                continue
            typed_manifest = cast("dict[str, object]", manifest)
            name = typed_manifest.get("name")
            if isinstance(name, str):
                manifest_names.append(name)
        return manifest_names

    return []
