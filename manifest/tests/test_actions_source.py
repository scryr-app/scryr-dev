"""GitHub polling declarations and compatibility with event-file reporting."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from scryr import CICD, ActionsReportSource, Github, Manifest, run_manifest_file


def test_legacy_source_and_defaults_roundtrip() -> None:
    """Existing report sources remain valid without opting into filename polling."""
    legacy = {"workflow_id": 42, "jobs_file": "results/jobs.json", "branch": "main"}
    source = ActionsReportSource.model_validate(legacy)
    assert source.model_dump() == {**legacy, "workflows": []}
    assert ActionsReportSource().model_dump() == {
        "workflow_id": None,
        "workflows": [],
        "jobs_file": None,
        "branch": None,
    }
    first = ActionsReportSource()
    first.workflows.append("ci.yml")
    assert ActionsReportSource().workflows == []


def test_filename_source_serializes_through_manifest() -> None:
    """Workflow filenames survive the exact artifact format consumed by the CLI."""
    manifest = Manifest(
        manifest_id="services/api",
        github=Github(repo_url="https://github.com/example/api"),
        cicd=CICD(
            platform="github_actions",
            source=ActionsReportSource(workflows=["ci.yml", "integration-tests.yaml"]),
        ),
    )
    serialized = manifest.to_dict()
    assert serialized["cicd"]["source"]["workflows"] == ["ci.yml", "integration-tests.yaml"]
    assert serialized["cicd"]["source"]["workflow_id"] is None
    assert serialized["manifestId"] == "services/api"
    assert Manifest.model_validate(serialized).to_dict() == serialized
    schema = Manifest.model_json_schema()["$defs"]["ActionsReportSource"]
    assert schema["properties"]["workflows"]["items"]["type"] == "string"
    assert "workflows" not in schema.get("required", [])


@pytest.mark.parametrize(
    "workflow",
    [
        "",
        "ci",
        "42",
        "ci.txt",
        "../ci.yml",
        ".github/workflows/ci.yml",
        "/ci.yml",
        "ci\\other.yml",
        "ci.yml?branch=main",
        "ci.yml#fragment",
        "ci%2fyml.yml",
        "ci..yml",
        " ci.yml",
        "ci.yml\n",
    ],
)
def test_reject_invalid_workflow_filenames(workflow: str) -> None:
    """Path traversal and URL fragments cannot enter provider endpoint paths."""
    with pytest.raises(ValidationError, match=r"plain \.yml or \.yaml filenames"):
        ActionsReportSource(workflows=[workflow])


@pytest.mark.parametrize("branch", ["", " ", "\t\n"])
def test_reject_empty_branch(branch: str) -> None:
    """Repository default selection is explicit omission, not an empty string."""
    with pytest.raises(ValidationError, match="branch must be nonempty"):
        ActionsReportSource(branch=branch)


def test_reject_ambiguous_workflow_selection() -> None:
    """Numeric workflow reporting remains usable, but cannot conflict with filenames."""
    with pytest.raises(ValidationError, match="mutually exclusive"):
        ActionsReportSource(workflow_id=42, workflows=["ci.yml"])
    assert ActionsReportSource(workflow_id=42, workflows=[]).workflow_id == 42
    assert (
        ActionsReportSource(workflows=["CI_v2.3.yml"], branch="release/v2").branch == "release/v2"
    )


def test_manifest_runtime_exports_polling_source(tmp_path) -> None:
    """The .scry importer preserves the native collector's declaration end to end."""
    entrypoint = tmp_path / "index.scry"
    entrypoint.write_text(
        "from scryr import ActionsReportSource, CICD, Diagram, Github, Manifest\n"
        'api = Manifest(manifest_id="services/api", name="API",\n'
        '    github=Github(repo_url="https://github.com/example/api"),\n'
        '    cicd=CICD(source=ActionsReportSource(workflows=["ci.yml", "test.yaml"])))\n'
        'diagram = Diagram(name="System", manifests=[api])\n'
    )
    records = run_manifest_file(entrypoint, "--json")
    assert isinstance(records, list)
    manifest = Manifest.model_validate(
        next(record["manifest"] for record in records if record["variable_name"] == "api")
    )
    assert manifest.cicd is not None
    assert manifest.cicd.source is not None
    assert manifest.cicd.source.workflows == ["ci.yml", "test.yaml"]
    assert manifest.manifest_id == "services/api"
