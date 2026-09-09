"""Report GitHub workflow events or poll recent runs into a Scryr Manifest's history."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path

from .action_client import GithubActionsClient, ScryrClient
from .github import GithubActionRun


def main(argv: list[str] | None = None) -> int:
    """Run an explicit reporting command; never execute code from event payloads."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-id", required=True)
    parser.add_argument("--endpoint", default=os.environ.get("SCRYR_ENDPOINT"), required=False)
    parser.add_argument("--clerk-org-id", default=os.environ.get("SCRYR_CLERK_ORG_ID"))
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--event-file", type=Path, help="GitHub workflow_run event JSON")
    mode.add_argument("--repository", help="Poll owner/name through GitHub's REST API")
    parser.add_argument("--event-id", help="Webhook delivery ID, when available")
    parser.add_argument("--workflow-id", type=int)
    parser.add_argument("--branch")
    parser.add_argument("--limit", type=int, default=100)
    args = parser.parse_args(argv)
    if not args.endpoint:
        parser.error("--endpoint or SCRYR_ENDPOINT is required")
    client = ScryrClient(
        args.endpoint,
        token=os.environ.get("SCRYR_TOKEN"),
        clerk_org_id=args.clerk_org_id,
    )
    if args.event_file:
        payload = json.loads(args.event_file.read_text())
        if "workflow_run" not in payload:
            parser.error("--event-file must contain a workflow_run event")
        run = GithubActionRun.from_github(payload)
        client.record_action_run(args.manifest_id, run, event_id=args.event_id, source="webhook")
    else:
        token = os.environ.get("GITHUB_TOKEN")
        if not token:
            parser.error("GITHUB_TOKEN is required for polling")
        github = GithubActionsClient(
            token,
            api_url=os.environ.get("GITHUB_API_URL", "https://api.github.com"),
        )
        for run in github.iter_runs(
            args.repository,
            workflow_id=args.workflow_id,
            branch=args.branch,
            limit=args.limit,
        ):
            client.record_action_run(args.manifest_id, run)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
