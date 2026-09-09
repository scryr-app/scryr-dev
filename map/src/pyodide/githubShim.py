"""Offline browser counterparts of the SDK's workflow history models."""

import copy
import hashlib
import json
from datetime import datetime, timezone
from urllib.parse import urlsplit


def _camel(name):
    first, *rest = name.split("_")
    return first + "".join(part.title() for part in rest)


def _wire(value):
    if isinstance(value, datetime):
        return value.isoformat().replace("+00:00", "Z")
    if isinstance(value, list):
        return [_wire(item) for item in value]
    if hasattr(value, "to_dict"):
        return value.to_dict()
    return value


class ActionStatusEvent:
    def __init__(self, **values):
        self.__dict__.update(values)

    @property
    def order_key(self):
        rank = {"completed": 2, "in_progress": 1}.get(self.status, 0)
        return self.source_updated_at, rank, self.event_id

    def to_dict(self):
        return {_camel(key): _wire(value) for key, value in self.__dict__.items()}


class GithubActionRun:
    def __init__(self, **values):
        defaults = dict(host="github.com", run_attempt=1, head_branch=None, conclusion=None,
                        run_started_at=None, logs_url=None, events=[])
        self.__dict__.update({**defaults, **values})
        for name in ("created_at", "updated_at", "run_started_at"):
            value = getattr(self, name)
            if isinstance(value, str):
                value = datetime.fromisoformat(value.replace("Z", "+00:00"))
                setattr(self, name, value)
            if value is not None and value.tzinfo is None:
                raise ValueError("Workflow timestamps must include a timezone")
        if self.status not in {"queued", "in_progress", "completed", "waiting", "pending", "requested"}:
            raise ValueError("Invalid workflow status")
        if self.status != "completed" and self.conclusion is not None:
            raise ValueError("Only completed runs can have a conclusion")

    @property
    def identity(self):
        return self.host.lower(), self.repository_id, self.run_id, self.run_attempt

    @classmethod
    def from_github(cls, payload):
        run = payload.get("workflow_run", payload)
        repo = run.get("repository") or payload.get("repository")
        return cls(
            host=urlsplit(run["html_url"]).netloc.lower(), repository_id=repo["id"],
            repository=repo["full_name"], workflow_id=run["workflow_id"],
            workflow_name=run.get("name") or "", run_id=run["id"],
            run_attempt=run.get("run_attempt", 1), head_branch=run.get("head_branch"),
            head_sha=run["head_sha"], html_url=run["html_url"], status=run["status"],
            conclusion=run.get("conclusion"), created_at=run["created_at"],
            updated_at=run["updated_at"], run_started_at=run.get("run_started_at"),
            logs_url=run.get("logs_url"),
        )

    def observation(self, *, event_id=None, source="api", recorded_at=None):
        fingerprint = json.dumps([*self.identity, self.status, self.conclusion,
                                  self.updated_at.astimezone(timezone.utc).isoformat()])
        return ActionStatusEvent(
            event_id=event_id or hashlib.sha256(fingerprint.encode()).hexdigest(),
            status=self.status, conclusion=self.conclusion, source_updated_at=self.updated_at,
            recorded_at=recorded_at or datetime.now(timezone.utc), source=source,
        )

    def to_dict(self):
        return {_camel(key): _wire(value) for key, value in self.__dict__.items()}


class GithubActionsLog:
    def __init__(self, *, runs=None):
        self.runs = list(runs or [])

    def record(self, run, *, event_id=None, source="api", recorded_at=None):
        event = run.observation(event_id=event_id, source=source, recorded_at=recorded_at)
        current = next((item for item in self.runs if item.identity == run.identity), None)
        if current is None:
            current = copy.deepcopy(run)
            current.events = [event]
            self.runs.append(current)
        else:
            if not current.events:
                current.events = [current.observation(source="manual")]
            if any(item.event_id == event.event_id or (
                item.status, item.conclusion, item.source_updated_at
            ) == (event.status, event.conclusion, event.source_updated_at) for item in current.events):
                return False
            latest = max(current.events, key=lambda item: item.order_key, default=None)
            if source == "api" and latest is not None and (
                event.source_updated_at >= latest.source_updated_at
                and (latest.status, latest.conclusion) == (event.status, event.conclusion)
            ):
                return False
            current.events.append(event)
            current.events.sort(key=lambda item: item.order_key)
            if max(current.events, key=lambda item: item.order_key) == event:
                replacement = copy.deepcopy(run)
                replacement.events = current.events
                self.runs[self.runs.index(current)] = replacement
        self.runs.sort(key=lambda run: (run.created_at, run.run_id, run.run_attempt), reverse=True)
        return True

    def build_status(self, *, repository, workflow_id=None, branch=None, host="github.com"):
        runs = [run for run in self.runs if run.host.lower() == host.lower()
                and run.repository == repository and (workflow_id is None or run.workflow_id == workflow_id)
                and (branch is None or run.head_branch == branch)]
        if not runs:
            return None
        latest = max(runs, key=lambda run: (run.created_at, run.run_id, run.run_attempt))
        if latest.status != "completed":
            return "pending"
        if latest.conclusion == "success":
            return "passing"
        if latest.conclusion in {"failure", "timed_out", "action_required", "startup_failure"}:
            return "failing"
        return None

    def to_dict(self):
        return {"runs": [run.to_dict() for run in self.runs]}
