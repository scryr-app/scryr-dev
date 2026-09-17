"""SDK boundary tests for typed inert collector declarations and wire contracts."""

from __future__ import annotations

import json
from datetime import timedelta
from pathlib import Path
from typing import TYPE_CHECKING

import pytest
from pydantic import ValidationError

import scryr
from scryr import Manifest, run_manifest_file
from scryr.collectors import (
    BiomeCheckCollector,
    CargoClippyCollector,
    CoberturaCoverageCollector,
    Command,
    CounterRate,
    DockerStatsCollector,
    EnvRef,
    GaugeSeries,
    GitHubActionsCollector,
    GitHubPullRequestCollector,
    GitStatusCollector,
    GrantLicenseCollector,
    GrypeScanCollector,
    HistogramPercentile,
    HyperfineBenchmarkCollector,
    JUnitReportCollector,
    LcovCoverageCollector,
    LicensePolicy,
    MiseTaskCollector,
    NextestCollector,
    OpenMetricsCollector,
    PytestCollector,
    RuffCheckCollector,
    SbomRef,
    Schedule,
    SyftInventoryCollector,
    ToolRequirement,
    VitestCollector,
)
from scryr.collectors.validation import validate_collector_references

if TYPE_CHECKING:
    from pydantic import BaseModel


@pytest.mark.parametrize(
    ("section", "collector"),
    [
        ("repository", GitStatusCollector()),
        ("repository", GitHubPullRequestCollector(repository="owner/repo")),
        ("repository", GitHubActionsCollector(repository="owner/repo")),
        ("checks", RuffCheckCollector()),
        ("checks", BiomeCheckCollector()),
        ("checks", CargoClippyCollector()),
        ("checks", MiseTaskCollector(task="verify")),
        ("metrics", OpenMetricsCollector(endpoint="http://127.0.0.1:8080/metrics")),
        ("metrics", DockerStatsCollector(containers=["api"])),
        ("tests", PytestCollector()),
        ("tests", VitestCollector()),
        ("tests", NextestCollector()),
        ("tests", JUnitReportCollector(files=["reports/junit.xml"])),
        ("tests", LcovCoverageCollector(files=["coverage/lcov.info"])),
        ("tests", CoberturaCoverageCollector(files=["coverage.xml"])),
        ("dependencies", SyftInventoryCollector()),
        ("performance", HyperfineBenchmarkCollector(command=Command(executable="python"))),
    ],
)
def test_concrete_collectors_round_trip_only_in_their_section(
    section: str, collector: BaseModel
) -> None:
    """Every integration is typed; wrong-section placement fails at the source boundary."""
    encoded = collector.model_dump(mode="json")
    manifest = Manifest.model_validate({"manifestId": "api", section: [encoded]})
    assert manifest.to_dict()[section] == [encoded]
    assert encoded["id"] == encoded["kind"]
    assert encoded["timeout"] == 300
    assert encoded["freshness"] == 3600
    assert encoded["env"] == {}
    assert Manifest.model_validate_json(manifest.model_dump_json()) == manifest
    wrong = "checks" if section != "checks" else "repository"
    with pytest.raises(ValidationError):
        Manifest.model_validate({"manifestId": "api", wrong: [encoded]})


def test_timedelta_wire_is_seconds_and_restores() -> None:
    """Python-friendly durations use the same numeric units as Rust."""
    collector = GitStatusCollector(
        schedule=Schedule(every=timedelta(seconds=2.5), debounce=timedelta(milliseconds=200)),
        timeout=timedelta(seconds=12),
    )
    wire = collector.model_dump(mode="json")
    assert wire["schedule"]["every"] == 2.5
    assert wire["schedule"]["debounce"] == 0.2
    assert wire["timeout"] == 12
    assert GitStatusCollector.model_validate(wire) == collector
    assert (
        GitStatusCollector.model_json_schema(mode="serialization")["$defs"]["Duration"]["type"]
        == "number"
    )


def test_defaults_and_mutable_fields_are_isolated() -> None:
    """Integration defaults are deterministic and do not share mutable state."""
    first, second = GitStatusCollector(), GitStatusCollector()
    assert first.schedule.every == timedelta(seconds=15)
    first.schedule.watch.append("src/**")
    assert second.schedule.watch == []
    assert GitHubActionsCollector(repository="o/r").schedule.every == timedelta(minutes=3)
    assert OpenMetricsCollector(endpoint="http://localhost/metrics").schedule.every == timedelta(
        seconds=30
    )
    assert PytestCollector().schedule == Schedule(manual=True)
    report = JUnitReportCollector(files=["reports/*.xml"])
    assert report.schedule.watch == ["reports/*.xml"]
    assert report.schedule.startup
    assert GrypeScanCollector().schedule.every == timedelta(days=1)
    assert GrantLicenseCollector().schedule.upstream_changed


@pytest.mark.parametrize(
    "fields", [{"manual": False}, {"every": -1}, {"debounce": -1}, {"every": 0}]
)
def test_invalid_schedules_fail(fields: dict[str, object]) -> None:
    """Disabled or negative schedules cannot enter the runtime plan."""
    with pytest.raises(ValidationError):
        Schedule.model_validate(fields)


@pytest.mark.parametrize(
    "path", ["../outside", "/outside/project", "C:/outside", "sub/../../outside", "bad\x00path"]
)
def test_paths_cannot_escape_project_lexically(path: str) -> None:
    """Declaration validation is filesystem-free; runtime must additionally check symlinks."""
    with pytest.raises(ValidationError):
        GitStatusCollector(directory=path)


def test_manifest_requires_stable_identity_and_unique_section_ids() -> None:
    """Display names/list positions never become durable collector identities."""
    with pytest.raises(ValidationError, match="manifest_id"):
        Manifest(repository=[GitStatusCollector()])
    with pytest.raises(ValidationError, match="Duplicate collector"):
        Manifest(manifest_id="api", repository=[GitStatusCollector(), GitStatusCollector()])
    manifest = Manifest(
        manifest_id="api", repository=[GitStatusCollector(id="a"), GitStatusCollector(id="b")]
    )
    before = {c.id for c in manifest.repository}
    manifest.repository.reverse()
    assert {c.id for c in manifest.repository} == before
    assert Manifest().to_dict()["repository"] == []


def test_dependency_pipeline_refs_and_policy_are_typed() -> None:
    """License and vulnerability evidence consume the same declared inventory."""
    manifest = Manifest(
        manifest_id="api",
        dependencies=[
            SyftInventoryCollector(id="packages"),
            GrantLicenseCollector(
                sbom=SbomRef(collector_id="packages"),
                policy=LicensePolicy(allow=["MIT", "Apache-2.0 OR MIT"]),
            ),
            GrypeScanCollector(sbom=SbomRef(collector_id="packages")),
        ],
    )
    validate_collector_references([manifest])
    data = manifest.to_dict()["dependencies"]
    assert data[1]["policy"]["unknown"] == "review"
    assert data[1]["sbom"] == data[2]["sbom"]
    with pytest.raises(ValidationError, match="both allowed and denied"):
        LicensePolicy(allow=["MIT"], deny=["MIT"])


def test_ambiguous_missing_and_wrong_sbom_producers_fail() -> None:
    """SBOM edges can only target Syft, which also excludes dependency cycles."""
    for dependencies in [
        [GrypeScanCollector()],
        [SyftInventoryCollector(id="a"), SyftInventoryCollector(id="b"), GrypeScanCollector()],
        [
            SyftInventoryCollector(),
            GrantLicenseCollector(id="policy"),
            GrypeScanCollector(sbom=SbomRef(collector_id="policy")),
        ],
    ]:
        with pytest.raises(ValidationError, match="exactly one Syft"):
            Manifest.model_validate({"manifestId": "api", "dependencies": dependencies})


def test_cross_manifest_refs_and_duplicate_manifest_identity() -> None:
    """The whole envelope validates cross-manifest producers before any execution."""
    packages = Manifest(manifest_id="packages", dependencies=[SyftInventoryCollector()])
    app = Manifest(
        manifest_id="app",
        dependencies=[
            GrypeScanCollector(sbom=SbomRef(manifest_id="packages", collector_id="syft_inventory"))
        ],
    )
    validate_collector_references([packages, app])
    with pytest.raises(ValueError, match="missing manifest"):
        validate_collector_references([app])
    with pytest.raises(ValueError, match="Duplicate manifest_id"):
        validate_collector_references([packages, Manifest(manifest_id="packages")])


def test_runtime_rejects_unresolved_cross_manifest_reference(tmp_path: Path) -> None:
    """CLI export/native and browser adapters validate the full source envelope."""
    entry = tmp_path / "index.scry"
    entry.write_text(
        "from scryr import Manifest\n"
        "from scryr.collectors import GrypeScanCollector, SbomRef\n"
        'api = Manifest(manifest_id="api", dependencies=[GrypeScanCollector(\n'
        '    sbom=SbomRef(manifest_id="missing", collector_id="syft_inventory"))])\n'
    )
    with pytest.raises(ValueError, match="missing manifest"):
        run_manifest_file(entry)


def test_env_ref_contains_names_never_machine_secrets(monkeypatch: pytest.MonkeyPatch) -> None:
    """Serialization does not resolve environment variables."""
    monkeypatch.setenv("EXAMPLE_TOKEN", "private-secret-value")
    collector = OpenMetricsCollector(
        endpoint="http://localhost/metrics",
        auth=EnvRef(name="EXAMPLE_TOKEN"),
        env={"API_TOKEN": EnvRef(name="EXAMPLE_TOKEN")},
    )
    assert "private-secret-value" not in collector.model_dump_json()
    assert collector.model_dump()["auth"] == {"name": "EXAMPLE_TOKEN"}
    with pytest.raises(ValidationError):
        EnvRef.model_validate({"name": "TOKEN", "value": "private"})
    with pytest.raises(ValidationError):
        ToolRequirement.model_validate({"executable": "different-scanner"})


def test_openmetrics_selectors_validate_format_and_ranges() -> None:
    """Metric selectors are concrete types, not a query dictionary or provider name."""
    collector = OpenMetricsCollector(
        endpoint="http://localhost/metrics",
        series=[
            CounterRate(metric="requests_total"),
            GaugeSeries(metric="memory_bytes"),
            HistogramPercentile(metric="duration_seconds", percentile=95),
        ],
    )
    assert [s["kind"] for s in collector.model_dump(mode="json")["series"]] == [
        "counter_rate",
        "gauge",
        "histogram_percentile",
    ]
    for percentile in [0, 101, float("nan")]:
        with pytest.raises(ValidationError):
            HistogramPercentile(metric="duration_seconds", percentile=percentile)
    for endpoint in [
        "file:///outside/metrics",
        "https://token@example.com/metrics",
        "https://example.com/#fragment",
    ]:
        with pytest.raises(ValidationError):
            OpenMetricsCollector(endpoint=endpoint)


def test_obsolete_sdk_api_and_provider_fields_are_removed() -> None:
    """The new contract is a breaking replacement, with no aliases or static fallbacks."""
    for name in [
        "Github",
        "CICD",
        "Metrics",
        "Tests",
        "Dependencies",
        "Performance",
        "PrometheusSource",
        "PostHogSource",
        "ScryrClient",
    ]:
        assert not hasattr(scryr, name)
    for field in ["github", "cicd", "analytics", "repo_url", "cicd_tool", "collectors"]:
        with pytest.raises(ValidationError):
            Manifest.model_validate({field: []})


def test_all_samples_execute_inertly(monkeypatch: pytest.MonkeyPatch) -> None:
    """Every maintained sample serializes without tools or SDK network clients."""
    import socket
    import subprocess

    def forbidden(*_args: object, **_kwargs: object) -> None:
        pytest.fail("Collector declaration attempted a process or network operation")

    monkeypatch.setattr(subprocess, "Popen", forbidden)
    monkeypatch.setattr(socket, "create_connection", forbidden)
    samples = Path(__file__).parent / "samples"
    for entry in sorted(samples.glob("*/index.scry")):
        payload = run_manifest_file(entry)
        assert json.loads(json.dumps(payload)) == payload
        assert run_manifest_file(entry, "--types")
