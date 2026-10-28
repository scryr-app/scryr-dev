"""Public package API contract tests."""

from __future__ import annotations

import json
from pathlib import Path

import scryr
import scryr.manifest
from scryr.cli import main


def test_top_level_package_exports_core_manifest_api() -> None:
    """Consumers can import the common model, enum, and runtime helpers from scryr."""
    forge = scryr.Forge(
        name="Package Forge",
        tools={"python": scryr.ForgeTool(version="3.14")},
    )
    manifest = scryr.Manifest(
        name="PackageAPI",
        info=scryr.Info(
            languages=[scryr.ProgrammingLanguage.python],
            frameworks=[scryr.WebFramework.fastapi],
        ),
        forges=[forge.name],
    )
    diagram = scryr.Diagram(name="Package Diagram", manifests=[manifest])
    query = scryr.ManifestQuery(languages=[scryr.ProgrammingLanguage.python])

    assert forge.to_mise_dict()["tools"]["python"]["version"] == "3.14"
    assert manifest.to_dict()["name"] == "PackageAPI"
    assert diagram.to_dict()["manifests"][0]["name"] == "PackageAPI"
    assert query.matches(manifest)
    assert scryr.Diagram.Query is scryr.ManifestQuery
    assert manifest.to_dict()["forges"] == ["Package Forge"]
    assert scryr.Info is scryr.Manifest.Info
    assert scryr.parse_version("1.2.3") == scryr.SemVer("1.2.3")
    assert callable(scryr.run_manifest_file)
    for name in (
        "Info",
        "OtherDiagram",
    ):
        model = getattr(scryr, name)
        assert model.__name__ == name
        assert not hasattr(scryr.manifest, name + "ManifestSection")


def test_console_main_returns_usage_error_without_arguments(capsys) -> None:
    """The packaged console script fails clearly when called without a manifest path."""
    exit_code = main([])

    captured = capsys.readouterr()
    assert exit_code == 2
    assert "Usage: scryr" in captured.err


def test_run_manifest_file_is_public_library_entrypoint() -> None:
    """run_manifest_file loads a manifest file without importing from the CLI module."""
    manifest_dir = Path(__file__).parent.parent
    payload = scryr.run_manifest_file(
        manifest_dir / "tests" / "samples" / "open_saas" / "index.scry",
        "--json",
    )

    assert isinstance(payload, list)
    assert json.loads(json.dumps(payload)) == payload


def test_removed_sections_are_not_public_or_serialized() -> None:
    """Removed designs are absent from exports, schema, and generated manifests."""
    import pytest
    from pydantic import ValidationError

    manifest = scryr.Manifest(name="API")
    for name in ("Github", "Metrics", "CICD", "Tests", "Dependencies", "Performance"):
        assert not hasattr(scryr, name)
        assert not hasattr(scryr.Manifest, name)
        assert name not in scryr.Manifest.model_json_schema().get("$defs", {})
        assert name.lower() not in manifest.to_dict()
        with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
            scryr.Manifest.model_validate({"name": "API", name.lower(): {}})
    for name in ("repo_url", "cicd_tool"):
        with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
            scryr.Manifest.model_validate({"name": "API", name: "legacy"})
