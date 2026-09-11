"""Workflow history behavior and provider/wire contracts."""

from __future__ import annotations

import json
import runpy
from datetime import UTC, datetime
from pathlib import Path

import pytest
from pydantic import ValidationError

from scryr import (
    CICD,
    GithubActionRun,
    GithubActionsLog,
    Manifest,
    ScryrClient,
    WebFramework,
    run_manifest_file,
)
from scryr.action_client import GithubActionsClient
from scryr.actions import main


def payload(**updates: object) -> dict:
    """Return a representative GitHub REST run."""
    return {
        "id": 12345,
        "run_attempt": 1,
        "workflow_id": 42,
        "name": "CI",
        "repository": {"id": 123, "full_name": "example/api"},
        "head_branch": "main",
        "head_sha": "a" * 40,
        "html_url": "https://github.com/example/api/actions/runs/12345",
        "created_at": "2026-09-08T10:00:00Z",
        "updated_at": "2026-09-08T10:00:00Z",
        "status": "queued",
        "conclusion": None,
        **updates,
    }


def test_roundtrip_history_and_short_schema_titles() -> None:
    """History survives Manifest JSON serialization with concise schema definitions."""
    log = GithubActionsLog()
    log.record(GithubActionRun.from_github(payload()))
    manifest = Manifest(
        manifest_id="services/api",
        frameworks=[WebFramework.fastapi],
        cicd=CICD(github_actions=log),
    )
    serialized = manifest.to_dict()
    assert serialized["manifestId"] == "services/api"
    assert serialized["cicd"]["githubActions"]["runs"][0]["runId"] == 12345
    assert Manifest.model_validate(serialized).to_dict() == serialized
    schema = Manifest.model_json_schema()
    assert schema["$defs"]["CICD"]["title"] == "CICD"
    assert "ManifestSection" not in json.dumps(schema)


def test_duplicates_late_deliveries_and_reruns() -> None:
    """Duplicate polling is ignored and late events cannot replace completion."""
    log = GithubActionsLog()
    completed = GithubActionRun.from_github(
        payload(
            status="completed",
            conclusion="failure",
            updated_at="2026-09-08T10:02:00Z",
        )
    )
    assert log.record(completed, event_id="delivery")
    assert not log.record(completed, event_id="delivery")
    assert not log.record(
        GithubActionRun.from_github(
            payload(
                status="completed",
                conclusion="failure",
                updated_at="2026-09-08T10:03:00Z",
            )
        )
    )
    assert log.record(GithubActionRun.from_github(payload()))
    assert log.record(
        GithubActionRun.from_github(
            payload(
                status="in_progress",
                updated_at="2026-09-08T10:01:00Z",
            )
        )
    )
    assert log.runs[0].status == "completed"
    assert [event.status for event in log.runs[0].events] == ["queued", "in_progress", "completed"]
    assert completed.events == []
    assert log.record(GithubActionRun.from_github(payload(run_attempt=2)))
    assert len(log.runs) == 2
    assert log.runs[0].run_attempt == 2


def test_same_timestamp_prefers_completed_and_keeps_hosts_separate() -> None:
    """Timestamp ties have a stable phase order and enterprise runs have their own key."""
    log = GithubActionsLog()
    log.record(GithubActionRun.from_github(payload(status="completed", conclusion="success")))
    log.record(GithubActionRun.from_github(payload(status="in_progress")))
    assert log.runs[0].status == "completed"
    log.record(
        GithubActionRun.from_github(
            payload(
                html_url="https://github.example.com/example/api/actions/runs/12345",
            )
        )
    )
    assert len(log.runs) == 2


def test_webhook_repository_fallback_and_timestamps() -> None:
    """Full webhook payloads preserve provider times and repository identity."""
    run = payload()
    repository = run.pop("repository")
    parsed = GithubActionRun.from_github({"workflow_run": run, "repository": repository})
    event = parsed.observation(recorded_at=datetime(2026, 9, 8, 11, tzinfo=UTC))
    assert event.source_updated_at.hour == 10
    assert event.recorded_at.hour == 11
    assert parsed.repository_id == 123


@pytest.mark.parametrize(
    "updates",
    [
        {"status": "passing"},
        {"status": "queued", "conclusion": "failure"},
        {"run_attempt": 0},
        {"updated_at": "2026-09-08T10:00:00"},
        {"updated_at": "2026-09-07T10:00:00Z"},
    ],
)
def test_reject_invalid_provider_states(updates) -> None:
    """Provider state and time fields are validated at the SDK boundary."""
    with pytest.raises(ValidationError):
        GithubActionRun.from_github(payload(**updates))


def test_build_summary_requires_explicit_workflow_and_branch() -> None:
    """Unrelated workflows do not affect selected build health."""
    log = GithubActionsLog()
    log.record(GithubActionRun.from_github(payload(status="completed", conclusion="success")))
    log.record(
        GithubActionRun.from_github(
            payload(
                id=99999,
                workflow_id=99,
                status="completed",
                conclusion="failure",
            )
        )
    )
    assert log.build_status(repository="example/api") == "failing"
    assert log.build_status(repository="example/api", workflow_id=42, branch="main") == "passing"
    assert log.build_status(repository="example/api", workflow_id=42, branch="other") is None


def test_client_paginates_without_changing_page_size(monkeypatch) -> None:
    """The last partial page must not change the offset implied by per_page."""
    urls = []

    def request(url, headers, body=None) -> dict:
        urls.append(url)
        return {"workflow_runs": [payload(id=index + 1) for index in range(100)]}

    monkeypatch.setattr("scryr.action_client._request_json", request)
    assert len(list(GithubActionsClient("token").iter_runs("example/api", limit=105))) == 105
    assert "per_page=100&page=2" in urls[1]


def test_reporter_and_client_wire_contract(monkeypatch, tmp_path) -> None:
    """A reporter sends organization context and no credentials in JSON."""
    requests = []

    def request(url, headers, body=None) -> dict:
        requests.append((url, headers, body))
        return {"data": {"recordActionRun": True}}

    monkeypatch.setattr("scryr.action_client._request_json", request)
    monkeypatch.setenv("SCRYR_TOKEN", "private-token")
    event = tmp_path / "event.json"
    event.write_text(json.dumps({"workflow_run": payload()}))
    assert (
        main(
            [
                "--manifest-id",
                "services/api",
                "--endpoint",
                "http://localhost:8000/graphql",
                "--clerk-org-id",
                "org",
                "--event-file",
                str(event),
            ]
        )
        == 0
    )
    _, headers, body = requests[0]
    assert headers["X-Scryr-Clerk-Org-Id"] == "org"
    assert body["variables"]["source"] == "webhook"
    assert body["variables"]["run"]["runAttempt"] == 1
    assert "private-token" not in json.dumps(body)
    assert "events" not in body["variables"]["run"]


def test_graphql_errors_are_reported(monkeypatch) -> None:
    """Authorization errors surface instead of being reported as successful writes."""
    monkeypatch.setattr(
        "scryr.action_client._request_json",
        lambda *_args: {
            "errors": [{"message": "permission denied"}],
        },
    )
    with pytest.raises(RuntimeError, match="permission denied"):
        ScryrClient("http://localhost/graphql").action_history("services/api")


def test_record_preserves_initial_run_without_explicit_events() -> None:
    """A log initialized with a run retains that state when an older event arrives."""
    current = GithubActionRun.from_github(
        payload(
            status="completed",
            conclusion="success",
            updated_at="2026-09-08T10:02:00Z",
        )
    )
    log = GithubActionsLog(runs=[current])
    log.record(GithubActionRun.from_github(payload()))
    assert log.runs[0].status == "completed"
    assert len(log.runs[0].events) == 2


def test_offline_sample_emits_a_status_timeline() -> None:
    """The shipped sample executes through the real Manifest runtime."""
    sample = Path(__file__).parent / "samples/github_actions/index.scry"
    records = run_manifest_file(sample, "--json")
    assert isinstance(records, list)
    manifest = Manifest.model_validate(
        next(record["manifest"] for record in records if record["variable_name"] == "api"),
    )
    from scryr import GitHubActionsPipeline

    assert manifest.manifest_id == "api"
    assert manifest.cards is not None
    pipeline = manifest.cards[0]
    assert isinstance(pipeline, GitHubActionsPipeline)
    assert pipeline.id == "github_pipeline"
    assert pipeline.observations is not None
    assert pipeline.observations.build_status(repository="example/api") == "passing"
    assert len(pipeline.observations.runs[0].events) == 3


def test_browser_history_matches_sdk() -> None:
    """The offline browser port retains the SDK wire format and event ordering."""
    browser_types = runpy.run_path(
        str(Path(__file__).parents[2] / "map/src/pyodide/githubShim.py"),
    )
    browser = browser_types["GithubActionsLog"]()
    sdk = GithubActionsLog()
    observations = [
        payload(status="completed", conclusion="success", updated_at="2026-09-08T10:02:00Z"),
        payload(),
        payload(status="in_progress", updated_at="2026-09-08T10:01:00Z"),
        payload(status="completed", conclusion="success", updated_at="2026-09-08T10:03:00Z"),
        payload(run_attempt=2),
    ]
    for observation in observations:
        recorded_at = datetime(2026, 9, 8, 11, tzinfo=UTC)
        assert browser.record(
            browser_types["GithubActionRun"].from_github(observation),
            recorded_at=recorded_at,
        ) == sdk.record(GithubActionRun.from_github(observation), recorded_at=recorded_at)
    assert browser.to_dict() == sdk.model_dump(mode="json")
