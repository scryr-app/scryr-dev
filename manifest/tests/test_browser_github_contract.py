"""Keep the legacy browser history shim compatible with the SDK contract."""

from __future__ import annotations

import runpy
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import pytest

from scryr import GithubActionRun, GithubActionsLog


def browser_types() -> dict[str, Any]:
    """Load the browser shim without importing browser runtime dependencies."""
    return runpy.run_path(str(Path(__file__).parents[2] / "map/src/pyodide/githubShim.py"))


def payload(**updates: object) -> dict[str, Any]:
    """Create a completed provider run with stable IDs and timestamps."""
    return {
        "id": 1,
        "workflow_id": 10,
        "name": "CI",
        "repository": {"id": 123, "full_name": "example/api"},
        "head_branch": "main",
        "head_sha": "a" * 40,
        "html_url": "https://github.com/example/api/actions/runs/1",
        "created_at": "2026-09-17T10:00:00Z",
        "updated_at": "2026-09-17T10:01:00Z",
        "status": "completed",
        "conclusion": "success",
        **updates,
    }


@pytest.mark.parametrize(
    "observation",
    [
        payload(),
        payload(path=".github/workflows/ci.yml"),
        {"workflow_run": payload(), "workflow": {"path": ".github/workflows/ci.yml"}},
    ],
)
def test_browser_workflow_path_wire_parity(observation: dict[str, Any]) -> None:
    """Optional workflow paths use the same camelCase and omission rules."""
    browser = browser_types()["GithubActionRun"].from_github(observation)
    sdk = GithubActionRun.from_github(observation)
    assert browser.to_dict() == sdk.model_dump(mode="json")


@pytest.mark.parametrize(
    ("observations", "expected"),
    [
        ([payload(), payload(id=2, workflow_id=20, conclusion="failure")], "failing"),
        ([payload(), payload(id=2, workflow_id=20, status="queued", conclusion=None)], "pending"),
        (
            [
                payload(conclusion="failure"),
                payload(id=2, workflow_id=20, status="queued", conclusion=None),
            ],
            "failing",
        ),
        ([payload(), payload(id=2, workflow_id=20, conclusion="cancelled")], None),
        ([payload(conclusion="failure"), payload(run_attempt=2)], "passing"),
        ([payload(), payload(id=2, head_branch="feature", conclusion="failure")], "failing"),
        ([], None),
    ],
)
def test_browser_aggregate_workflow_status(
    observations: list[dict[str, Any]], expected: str | None
) -> None:
    """A passing workflow or older retry cannot conceal another selected failure."""
    types = browser_types()
    browser = types["GithubActionsLog"](
        runs=[types["GithubActionRun"].from_github(value) for value in observations]
    )
    sdk = GithubActionsLog(runs=[GithubActionRun.from_github(value) for value in observations])
    assert browser.build_status(repository="example/api") == expected
    assert sdk.build_status(repository="example/api") == expected
    assert browser.build_status(repository="example/api", branch="main") == sdk.build_status(
        repository="example/api", branch="main"
    )


def test_browser_poll_enrichment_keeps_path_without_inventing_events() -> None:
    """Late metadata enriches snapshots, survives later reports, and deduplicates."""
    types = browser_types()
    browser = types["GithubActionsLog"]()
    sdk = GithubActionsLog()
    recorded_at = datetime(2026, 9, 17, 11, tzinfo=UTC)
    observations = [
        payload(status="queued", conclusion=None),
        payload(status="queued", conclusion=None, path=".github/workflows/ci.yml"),
        payload(updated_at="2026-09-17T10:02:00Z"),
        payload(updated_at="2026-09-17T10:02:00Z"),
    ]
    for index, observation in enumerate(observations):
        browser_run = types["GithubActionRun"].from_github(observation)
        sdk_run = GithubActionRun.from_github(observation)
        if index == 1:
            browser_run.jobs = []
            sdk_run.jobs = []
        assert browser.record(browser_run, recorded_at=recorded_at) == sdk.record(
            sdk_run, recorded_at=recorded_at
        )
        assert browser.to_dict() == sdk.model_dump(mode="json")
    assert len(browser.runs[0].events) == 2
    assert browser.runs[0].workflow_path == ".github/workflows/ci.yml"
