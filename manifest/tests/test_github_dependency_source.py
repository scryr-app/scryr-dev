"""Public dependency collection declarations and manual-report compatibility."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

import scryr
import scryr.manifest
from scryr import Dependencies, Github, GithubDependencySource, Manifest, run_manifest_file


def test_public_source_defaults_and_opt_in() -> None:
    """Importing and enabling the provider are explicit, without changing manual sections."""
    assert "GithubDependencySource" in scryr.__all__
    assert GithubDependencySource is scryr.manifest.GithubDependencySource
    assert GithubDependencySource().model_dump() == {
        "provider": "github",
        "inventory": True,
        "security": True,
    }
    assert Dependencies().source is None
    assert Manifest(name="API").dependencies is None


@pytest.mark.parametrize(("inventory", "security"), [(True, True), (True, False), (False, True)])
def test_source_roundtrips_through_manifest(*, inventory: bool, security: bool) -> None:
    """Each useful collection mode survives the exact artifact wire contract."""
    manifest = Manifest(
        manifest_id="services/api",
        github=Github(repo_url="https://github.com/example/api"),
        dependencies=Dependencies(
            source=GithubDependencySource(inventory=inventory, security=security),
            total_deps=12,
        ),
    )
    wire = manifest.to_dict()
    assert wire["dependencies"]["source"] == {
        "provider": "github",
        "inventory": inventory,
        "security": security,
    }
    assert wire["dependencies"]["totalDeps"] == 12
    assert Manifest.model_validate(wire).to_dict() == wire


@pytest.mark.parametrize(
    ("value", "match"),
    [
        ({"provider": "gitlab"}, "Input should be 'github'"),
        ({"inventory": False, "security": False}, "requires inventory or security"),
        ({"branch": "main"}, "Extra inputs are not permitted"),
        ({"path": "services/api/package.json"}, "Extra inputs are not permitted"),
        ({"token": "not-a-real-token"}, "Extra inputs are not permitted"),
    ],
)
def test_reject_invalid_or_unsupported_source_settings(value, match: str) -> None:
    """No disabled source, other provider, package scope, branch, or token is accepted."""
    with pytest.raises(ValidationError, match=match):
        GithubDependencySource.model_validate(value)
    with pytest.raises(ValidationError, match=match):
        Dependencies.model_validate({"source": value})


def test_manual_dependency_fields_and_reports_remain_compatible() -> None:
    """Legacy aliases and durable report content retain their existing meaning."""
    fields = {
        "totalDeps": 12,
        "outdatedDeps": 3,
        "vulnerableDeps": 2,
        "maxSeverity": "high",
        "directDeps": 4,
        "transitiveDeps": 8,
        "openAlerts": 2,
        "updateLag": 5,
        "licenseCompliance": "compliant",
        "reports": {"dependencies": {"manual": {"runId": "build-42"}}},
    }
    manual = Dependencies.model_validate(fields)
    assert manual.source is None
    assert manual.model_dump(exclude_none=True) == fields
    with_source = Dependencies.model_validate({**fields, "source": {"security": False}})
    assert with_source.model_dump(exclude={"source"}, exclude_none=True) == fields
    assert with_source.source == GithubDependencySource(security=False)


def test_schema_adds_optional_source_without_requiring_legacy_data_changes() -> None:
    """Consumers can discover the provider while existing dependency objects stay valid."""
    schema = Manifest.model_json_schema()["$defs"]
    dependency_schema = schema["Dependencies"]
    assert "source" not in dependency_schema.get("required", [])
    assert dependency_schema["properties"]["source"]["default"] is None
    source_schema = schema["GithubDependencySource"]
    assert source_schema["additionalProperties"] is False
    assert source_schema["properties"]["provider"]["const"] == "github"
    for field in ("inventory", "security"):
        assert source_schema["properties"][field]["default"] is True
        assert source_schema["properties"][field]["type"] == "boolean"


def test_runtime_exports_repository_dependency_source(tmp_path) -> None:
    """A trusted .scry file imports the public source and emits a collector-ready manifest."""
    entrypoint = tmp_path / "index.scry"
    entrypoint.write_text(
        "from scryr import Dependencies, Diagram, Github, GithubDependencySource, Manifest\n"
        'api = Manifest(manifest_id="services/api", name="API",\n'
        '    github=Github(repo_url="https://github.com/example/api"),\n'
        "    dependencies=Dependencies(source=GithubDependencySource(inventory=False)))\n"
        'diagram = Diagram(name="System", manifests=[api])\n'
    )
    records = run_manifest_file(entrypoint, "--json")
    assert isinstance(records, list)
    manifest = Manifest.model_validate(
        next(record["manifest"] for record in records if record["variable_name"] == "api")
    )
    assert manifest.dependencies is not None
    assert manifest.dependencies.source == GithubDependencySource(inventory=False)
