"""GitHub Actions observations and deterministic status history (no network I/O)."""

from __future__ import annotations

import hashlib
import json
from datetime import UTC, datetime
from typing import Any, Literal
from urllib.parse import urlsplit

from pydantic import AwareDatetime, BaseModel, ConfigDict, Field, model_validator
from pydantic.alias_generators import to_camel

type ActionStatus = Literal["queued", "in_progress", "completed", "waiting", "pending", "requested"]
type ActionConclusion = Literal[
    "success",
    "failure",
    "cancelled",
    "neutral",
    "skipped",
    "stale",
    "timed_out",
    "action_required",
    "startup_failure",
]
type ActionSource = Literal["api", "webhook", "manual"]


class _ActionModel(BaseModel):
    """Shared JSON naming and validation rules."""

    model_config = ConfigDict(
        extra="forbid",
        alias_generator=to_camel,
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )


class ActionStatusEvent(_ActionModel):
    """An observed state, with provider time separate from collection time."""

    event_id: str = Field(min_length=1)
    status: ActionStatus
    conclusion: ActionConclusion | None = None
    source_updated_at: AwareDatetime
    recorded_at: AwareDatetime = Field(default_factory=lambda: datetime.now(UTC))
    source: ActionSource = "api"

    @model_validator(mode="after")
    def _validate_conclusion(self) -> ActionStatusEvent:
        if self.status != "completed" and self.conclusion is not None:
            msg = "Only completed runs can have a conclusion"
            raise ValueError(msg)
        return self

    @property
    def order_key(self) -> tuple[datetime, int, str]:
        """Resolve same-timestamp transitions deterministically."""
        rank = {"completed": 2, "in_progress": 1}.get(self.status, 0)
        return self.source_updated_at, rank, self.event_id


class GithubActionJob(_ActionModel):
    """One job from a complete workflow-attempt snapshot."""

    id: int = Field(gt=0)
    name: str = Field(min_length=1)
    status: ActionStatus
    conclusion: ActionConclusion | None = None
    started_at: AwareDatetime | None = None
    completed_at: AwareDatetime | None = None
    html_url: str = Field(pattern=r"^https?://")

    @model_validator(mode="after")
    def _validate_state(self) -> GithubActionJob:
        """Validate the state and provider timestamps."""
        if self.status != "completed" and self.conclusion is not None:
            msg = "Only completed jobs can have a conclusion"
            raise ValueError(msg)
        if self.started_at and self.completed_at and self.completed_at < self.started_at:
            msg = "completed_at must not precede started_at"
            raise ValueError(msg)
        return self


class GithubActionRun(_ActionModel):
    """One workflow run attempt, including its observed status transitions."""

    host: str = Field(default="github.com", min_length=1)
    repository_id: int = Field(gt=0)
    repository: str = Field(pattern=r"^[^/\s]+/[^/\s]+$")
    workflow_id: int = Field(gt=0)
    workflow_name: str
    run_id: int = Field(gt=0)
    run_attempt: int = Field(default=1, gt=0)
    head_branch: str | None = None
    head_sha: str = Field(min_length=1)
    html_url: str = Field(pattern=r"^https?://")
    status: ActionStatus
    conclusion: ActionConclusion | None = None
    created_at: AwareDatetime
    updated_at: AwareDatetime
    run_started_at: AwareDatetime | None = None
    logs_url: str | None = None
    events: list[ActionStatusEvent] = Field(default_factory=list)
    jobs: list[GithubActionJob] | None = Field(default=None, exclude_if=lambda value: value is None)

    @property
    def identity(self) -> tuple[str, int, int, int]:
        """Stable identity that keeps reruns and GitHub hosts separate."""
        return self.host.lower(), self.repository_id, self.run_id, self.run_attempt

    @model_validator(mode="after")
    def _validate_state(self) -> GithubActionRun:
        if self.status != "completed" and self.conclusion is not None:
            msg = "Only completed runs can have a conclusion"
            raise ValueError(msg)
        if self.updated_at < self.created_at:
            msg = "updated_at must not precede created_at"
            raise ValueError(msg)
        return self

    @classmethod
    def from_github(cls, payload: dict[str, Any]) -> GithubActionRun:
        """Convert a REST run or a complete workflow_run webhook payload."""
        run = payload.get("workflow_run", payload)
        repository = run.get("repository") or payload.get("repository")
        if not isinstance(repository, dict):
            msg = "GitHub payload must include repository metadata"
            raise TypeError(msg)
        return cls(
            host=urlsplit(run["html_url"]).netloc.lower(),
            repository_id=repository["id"],
            repository=repository["full_name"],
            workflow_id=run["workflow_id"],
            workflow_name=run.get("name") or "",
            run_id=run["id"],
            run_attempt=run.get("run_attempt", 1),
            head_branch=run.get("head_branch"),
            head_sha=run["head_sha"],
            html_url=run["html_url"],
            status=run["status"],
            conclusion=run.get("conclusion"),
            created_at=run["created_at"],
            updated_at=run["updated_at"],
            run_started_at=run.get("run_started_at"),
            logs_url=run.get("logs_url"),
        )

    def observation(
        self,
        *,
        event_id: str | None = None,
        source: ActionSource = "api",
        recorded_at: datetime | None = None,
    ) -> ActionStatusEvent:
        """Create a retry-stable observation without mutating this run."""
        fingerprint = json.dumps(
            [
                *self.identity,
                self.status,
                self.conclusion,
                self.updated_at.astimezone(UTC).isoformat(),
            ]
        )
        return ActionStatusEvent(
            event_id=event_id or hashlib.sha256(fingerprint.encode()).hexdigest(),
            status=self.status,
            conclusion=self.conclusion,
            source_updated_at=self.updated_at,
            recorded_at=recorded_at or datetime.now(UTC),
            source=source,
        )


class GithubActionsLog(_ActionModel):
    """A serializable collection of workflow attempts and observed transitions."""

    runs: list[GithubActionRun] = Field(default_factory=list)

    def record(
        self,
        run: GithubActionRun,
        *,
        event_id: str | None = None,
        source: ActionSource = "api",
        recorded_at: datetime | None = None,
    ) -> bool:
        """Record a new observation; return False for a duplicate or unchanged poll."""
        event = run.observation(event_id=event_id, source=source, recorded_at=recorded_at)
        current = next((item for item in self.runs if item.identity == run.identity), None)
        if current is None:
            current = run.model_copy(deep=True)
            current.events = [event]
            self.runs.append(current)
        else:
            if not current.events:
                current.events = [current.observation(source="manual")]
            if any(item.event_id == event.event_id for item in current.events):
                return False
            if any(
                (item.status, item.conclusion, item.source_updated_at)
                == (event.status, event.conclusion, event.source_updated_at)
                for item in current.events
            ):
                return False
            latest = max(current.events, key=lambda item: item.order_key, default=None)
            if (
                source == "api"
                and latest is not None
                and (
                    event.source_updated_at >= latest.source_updated_at
                    and (latest.status, latest.conclusion) == (event.status, event.conclusion)
                )
            ):
                return False
            current.events.append(event)
            current.events.sort(key=lambda item: item.order_key)
            winner = max(current.events, key=lambda item: item.order_key)
            if winner == event:
                replacement = run.model_copy(deep=True)
                replacement.events = current.events
                if replacement.jobs is None:
                    replacement.jobs = current.jobs
                self.runs[self.runs.index(current)] = replacement
        self.runs.sort(
            key=lambda item: (item.created_at, item.run_id, item.run_attempt), reverse=True
        )
        return True

    def build_status(
        self,
        *,
        repository: str,
        workflow_id: int | None = None,
        branch: str | None = None,
        host: str = "github.com",
    ) -> Literal["passing", "failing", "pending"] | None:
        """Summarize the latest matching observed build; cancellations have no summary."""
        candidates = [
            run
            for run in self.runs
            if (
                run.host.lower() == host.lower()
                and run.repository == repository
                and (workflow_id is None or run.workflow_id == workflow_id)
                and (branch is None or run.head_branch == branch)
            )
        ]
        if not candidates:
            return None
        latest = max(candidates, key=lambda run: (run.created_at, run.run_id, run.run_attempt))
        if latest.status != "completed":
            return "pending"
        if latest.conclusion == "success":
            return "passing"
        if latest.conclusion in {"failure", "timed_out", "action_required", "startup_failure"}:
            return "failing"
        return None
