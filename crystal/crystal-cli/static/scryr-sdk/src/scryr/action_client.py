"""Explicit HTTP clients for collecting and persisting workflow observations."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Any
from urllib.parse import quote, urlencode, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

from .github import ActionSource, GithubActionRun, GithubActionsLog

if TYPE_CHECKING:
    from collections.abc import Iterator


class _NoRedirect(HTTPRedirectHandler):
    """Do not forward authentication headers to redirect destinations."""

    def redirect_request(self, *_args: object, **_kwargs: object) -> None:
        """Reject redirects so bearer credentials stay at the configured endpoint."""
        return


def _request_json(
    url: str,
    headers: dict[str, str],
    payload: dict[str, Any] | None = None,
) -> dict[str, Any]:
    parsed = urlsplit(url)
    if parsed.scheme != "https" and not (
        parsed.scheme == "http" and parsed.hostname in {"localhost", "127.0.0.1", "::1"}
    ):
        msg = "Use HTTPS, or HTTP for a local Scryr server"
        raise ValueError(msg)
    data = None if payload is None else json.dumps(payload).encode()
    request = Request(url, data=data, headers={"Content-Type": "application/json", **headers})  # noqa: S310 - scheme validated above.
    with build_opener(_NoRedirect()).open(request, timeout=30) as response:
        result = json.load(response)
    if not isinstance(result, dict):
        msg = "Expected a JSON object from the server"
        raise TypeError(msg)
    return result


class GithubActionsClient:
    """Read GitHub workflow observations; credentials are never part of a Manifest."""

    def __init__(self, token: str, *, api_url: str = "https://api.github.com") -> None:
        """Configure a GitHub token with Actions read access."""
        self._api_url = api_url.rstrip("/")
        self._headers = {
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2026-03-10",
        }

    @staticmethod
    def _repository_path(repository: str) -> str:
        parts = repository.split("/")
        if len(parts) != 2 or not all(parts):
            msg = "repository must be owner/name"
            raise ValueError(msg)
        return "/".join(quote(part, safe="") for part in parts)

    def get_run(
        self,
        repository: str,
        run_id: int,
        *,
        attempt: int | None = None,
    ) -> GithubActionRun:
        """Fetch a run, optionally selecting a historical rerun attempt."""
        if run_id < 1 or (attempt is not None and attempt < 1):
            msg = "run_id and attempt must be positive"
            raise ValueError(msg)
        path = f"repos/{self._repository_path(repository)}/actions/runs/{run_id}"
        if attempt is not None:
            path += f"/attempts/{attempt}"
        return GithubActionRun.from_github(_request_json(f"{self._api_url}/{path}", self._headers))

    def iter_runs(
        self,
        repository: str,
        *,
        workflow_id: int | None = None,
        branch: str | None = None,
        limit: int = 100,
    ) -> Iterator[GithubActionRun]:
        """Poll recent runs with pagination; unobserved transitions are not invented."""
        if limit < 1:
            msg = "limit must be positive"
            raise ValueError(msg)
        path = f"repos/{self._repository_path(repository)}/actions"
        if workflow_id is not None:
            path += f"/workflows/{workflow_id}"
        page = 1
        remaining = limit
        while remaining:
            parameters: dict[str, str | int] = {"per_page": min(100, limit), "page": page}
            if branch is not None:
                parameters["branch"] = branch
            payload = _request_json(
                f"{self._api_url}/{path}/runs?{urlencode(parameters)}",
                self._headers,
            )
            runs = payload["workflow_runs"]
            for run in runs[:remaining]:
                yield GithubActionRun.from_github(run)
                remaining -= 1
            if len(runs) < parameters["per_page"]:
                break
            page += 1


class ScryrClient:
    """Read and append operational history through Crystal's authenticated API."""

    def __init__(
        self,
        endpoint: str,
        *,
        token: str | None = None,
        clerk_org_id: str | None = None,
    ) -> None:
        """Configure the GraphQL endpoint and existing Scryr authentication."""
        self._endpoint = endpoint
        self._headers: dict[str, str] = {}
        if token:
            self._headers["Authorization"] = f"Bearer {token}"
        if clerk_org_id:
            self._headers["X-Scryr-Clerk-Org-Id"] = clerk_org_id

    def _execute(self, query: str, variables: dict[str, Any]) -> dict[str, Any]:
        payload = _request_json(
            self._endpoint,
            self._headers,
            {"query": query, "variables": variables},
        )
        if payload.get("errors"):
            msg = "; ".join(error["message"] for error in payload["errors"])
            raise RuntimeError(msg)
        return payload["data"]

    def record_action_run(
        self,
        manifest_id: str,
        run: GithubActionRun,
        *,
        event_id: str | None = None,
        source: ActionSource = "api",
    ) -> bool:
        """Persist one observation atomically; retries return False if already recorded."""
        query = """
        mutation Record($manifestId: String!, $run: JSON!, $eventId: String, $source: String!) {
          recordActionRun(manifestId: $manifestId, run: $run, eventId: $eventId, source: $source)
        }
        """
        return self._execute(
            query,
            {
                "manifestId": manifest_id,
                "run": run.model_dump(mode="json", exclude={"events"}),
                "eventId": event_id,
                "source": source,
            },
        )["recordActionRun"]

    def action_history(
        self,
        manifest_id: str,
        *,
        limit: int = 100,
        offset: int = 0,
    ) -> GithubActionsLog:
        """Load a page of complete workflow attempts for attachment or export."""
        query = """
        query History($manifestId: String!, $limit: Int!, $offset: Int!) {
          actionHistory(manifestId: $manifestId, limit: $limit, offset: $offset)
        }
        """
        payload = self._execute(
            query, {"manifestId": manifest_id, "limit": limit, "offset": offset}
        )
        return GithubActionsLog.model_validate(payload["actionHistory"])
