# Scryr

Scryr is a Python package for defining architecture manifests as typed Pydantic
models. Applications can use it as a library, and tools can execute manifest
files through `python -m scryr.cli`.

```bash
pip install scryr
```

```python
from scryr import CICD, Github, Info, Manifest, ProgrammingLanguage, WebFramework

api = Manifest(
    name="Public API",
    info=Info(
        description="Customer-facing HTTP API",
        language=ProgrammingLanguage.python,
        frameworks=[WebFramework.fastapi],
    ),
    github=Github(repo_url="https://github.com/example/api"),
    cicd=CICD(platform="github_actions"),
)

print(api.to_dict())
```

The public `scryr` command is the Rust CLI. The Python SDK entrypoint remains
available as a module for tooling:

```bash
python -m scryr.cli architecture.py --json
python -m scryr.cli architecture.py --types
python -m scryr.cli architecture.py --schema
```

## GitHub Actions history

Sections use short model names: `Info`, `Github`, `CICD`, `Metrics`, `Tests`,
`Dependencies`, `Performance`, and `OtherDiagram`. These are the actual class
names and JSON Schema titles. Import them directly from `scryr` and assign them
to the corresponding lowercase Manifest fields (`info`, `github`, `cicd`, etc.).

```python
from scryr import CICD, GithubActionRun, GithubActionsLog, Manifest

api = Manifest(
    manifest_id="services/api",
    name="API",
    cicd=CICD(platform="github_actions", github_actions=GithubActionsLog()),
)

# payload is a GitHub REST run or a complete workflow_run webhook event.
run = GithubActionRun.from_github(payload)
api.cicd.github_actions.record(run, event_id=delivery_id, source="webhook")
```

`GithubActionRun` preserves GitHub's separate `status` and `conclusion`, repository
ID, workflow ID, run ID, attempt, branch, commit SHA, timestamps, and run/log URLs.
Each `ActionStatusEvent` records the provider timestamp and the collection time.
Duplicate observations and unchanged polls are ignored; delayed events remain in
the history without replacing newer state. Reruns are separate attempts. Only
observed transitions are recorded: polling cannot reconstruct intermediate states
that were never collected. Console log text and credentials are not embedded.

`manifest_id` is optional for ordinary Manifests and required for durable history.
Choose an organization-wide unique ID and retain it when renaming or moving the
Manifest. Reusing it intentionally shares history across maps; changing it starts
a separate history. Events may arrive before the Manifest is first published.

To preserve an SDK-only snapshot, serialize `api.to_dict()` as JSON and reload with
`Manifest.model_validate(json.loads(saved_json))`. Saving is explicit; generating
an `.scry` file does not perform network requests or persist runtime mutations.

For concurrent reporters and history that survives regeneration, use Crystal:

```python
import os
from scryr import ScryrClient

client = ScryrClient(
    "https://your-scryr-server/graphql",
    token=os.environ["SCRYR_TOKEN"],
    clerk_org_id=os.environ["SCRYR_CLERK_ORG_ID"],
)
client.record_action_run("services/api", run, event_id=delivery_id, source="webhook")
api.cicd.github_actions = client.action_history("services/api")
```

Crystal stores an append-only, organization-scoped event ledger in SQLite or
Turso, independently of generated artifacts. Writes use the same organization
permissions as Manifest uploads and atomically deduplicate concurrent deliveries.
Reads attach the latest 100 run attempts to `cicd.githubActions`. The GraphQL
`actionHistory(manifestId:, limit:, offset:)` query and matching client method
page through older attempts, including their complete observed history. The
`blocks` query exposes `manifestId` and `githubActions` fields.

Crystal derives `cicd.buildStatus` from the newest reported run across workflows
when durable history is attached. This replaces static build status when history exists.
For an offline SDK summary, select a repository; workflow and branch filters are optional:

```python
api.cicd.build_status = api.cicd.github_actions.build_status(
    repository="example/api",
)
```

A cancelled, skipped, neutral, stale, or completed-without-conclusion run returns
`None`; it is not reported as a successful build.

## Collecting observations

The native Rust CLI reads `GITHUB_EVENT_PATH` in a separate trusted workflow:

```bash
scryr report-action-status --manifest-id services/api
```

Set `SCRYR_ENDPOINT`, `SCRYR_CLERK_ORG_ID`, and `SCRYR_TOKEN` for Crystal.
Set `GITHUB_TOKEN` with Contents read permission for branch discovery. The reporter
checks whether `main` exists, then `master`, then uses the repository default branch.
Runs from other branches are skipped. `--branch` overrides this selection and avoids
GitHub branch lookups. `--workflow-id` optionally narrows reporting; by default all
observed workflows are eligible. Offline SDK summaries use the observations supplied
and do not query GitHub to discover branches.

History survives regeneration, duplicate deliveries are ignored, and older deliveries
do not replace a newer build. Open maps refresh their server data every 30 seconds to display the derived status.

For polling or reconciliation, supply `GITHUB_TOKEN` with Actions read access:

```bash
python -m scryr.actions --manifest-id services/api \
  --repository example/api --workflow-id 42 --branch main --limit 200
```

`GithubActionsClient.get_run(repository, run_id, attempt=2)` retrieves a specific
attempt. `iter_runs()` paginates recent runs; repeated polling does not fabricate
older rerun attempts. Set `GITHUB_API_URL` for GitHub Enterprise. A webhook
receiver can pass verified `workflow_run` payloads through the same adapter and
client, using the GitHub delivery ID. This SDK does not expose an unauthenticated
webhook listener; signature verification belongs in the receiving service.

Provider references: [workflow runs API](https://docs.github.com/en/rest/actions/workflow-runs)
and [workflow_run events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run).
