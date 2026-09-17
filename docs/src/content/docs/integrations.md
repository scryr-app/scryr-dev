---
title: Integrations
description: Connect Scryr to source, CI, tests, metrics, analytics, GraphQL, and developer tooling.
---

Scryr keeps the stable architecture declaration in `index.scry` and attaches current evidence through typed providers and reporting commands.

## GitHub and source links

Add the canonical repository and useful documentation or operational links to a manifest:

```python
from scryr import Github, Info, Link, Manifest

api = Manifest(
    name="Public API",
    github=Github(repo_url="https://github.com/acme/api"),
    info=Info(
        docs=["https://docs.acme.test/api"],
        links=[Link(site_name="Runbook", http_url="https://ops.acme.test/api")],
    ),
)
```

These links appear with the block, keeping navigation from architecture to implementation and operations direct.

## Poll GitHub Actions locally

The OSS CLI can collect workflow and job history using your existing GitHub CLI login. Install `gh` and authenticate with `gh auth login`, then select workflows on each block:

```python title="index.scry"
from scryr import CICD, ActionsReportSource, Diagram, Github, Manifest

api = Manifest(
    manifest_id="services/api",
    name="Public API",
    github=Github(repo_url="https://github.com/acme/api"),
    cicd=CICD(
        platform="github_actions",
        source=ActionsReportSource(
            workflows=["ci.yml", "integration.yml"],
            branch="main",
        ),
    ),
)
diagram = Diagram(name="Commerce", manifests=[api])
```

Use filenames from `.github/workflows`, without the directory prefix. Existing numeric `workflow_id` selections also work; choose either `workflow_id` or `workflows`. Omit `branch` to follow the repository's default branch.

```sh
scryr serve
scryr serve --poll 60
```

After loading a valid diagram, `serve` syncs immediately and polls configured providers every 300 seconds (five minutes). The general `--poll` option accepts 15–3600 seconds; bare `--poll` also uses 300 seconds. Polling runs independently of `--watch` and stops with the server. `--watch` refreshes declarations after source edits. `--no-poll` disables collection. `--server-only` and Clerk-authenticated serving do not use your local GitHub login.

The initial sync imports ten recent runs per selected workflow, including complete job snapshots. Later polls scan up to 100 recent runs per workflow, including reruns of older completed runs, and refresh previously observed active attempts. Job snapshots are paginated up to 2,000 jobs per attempt. Failed cycles back off up to one hour, then resume the requested interval after recovery. Shared selections are fetched once per cycle and attached to every matching stable `manifest_id`. Repeated observations are deduplicated in Scryr's durable history. Polling records observed states; it cannot recover transitions that happened between polls.

To sync once into a running Scryr server:

```sh
scryr sync github --manifest api --endpoint http://127.0.0.1:8000/graphql
```

`--path` and `--manifest-dir` select another manifest project. Without `--manifest`, all configured GitHub blocks are collected. One-shot sync attaches history to stable IDs; use `scryr push` or `scryr serve` to load the diagram itself.

Collection requires an explicit source selection, stable `manifest_id`, and `github.repo_url`. A repository URL alone does not enable polling, and projects without configured sources make no `gh` calls. Scryr invokes `gh api`; authentication stays with `gh`, and tokens are not copied into manifests or diagram data. Missing `gh`, authentication failures, and provider errors are reported while the last valid diagram and collected history remain available. Repository statistics such as stars and pull-request counts are not automatically collected.

## Poll dependency inventory and security alerts

Opt in on blocks with a stable `manifest_id` and a GitHub repository URL:

```python
from scryr import Dependencies, GithubDependencySource

# Add this section to the block's Manifest(...):
dependencies = Dependencies(source=GithubDependencySource())
```

`GithubDependencySource` enables repository-wide inventory and security collection.
Use `inventory=False` or `security=False` to disable one; at least one must remain enabled.
This data describes the repository's default branch, not packages owned exclusively by an
individual diagram block. Blocks referencing the same repository share each collection.
`scryr serve --poll 300` and `scryr sync github` collect it alongside configured workflows.
Each background provider has its own retry schedule, so an unavailable inventory does not
slow workflow or security updates.

The Dependencies card shows known package counts, open security alerts, unique vulnerable
packages, highest severity, and alert links with available patched versions. Package details
include versions and declared licenses when present. Direct and transitive counts appear only
when the dependency graph provides complete relationships. Inventory is not an audit of
outdated versions or license compliance, and no such values are inferred.

Scryr uses GitHub's asynchronous SBOM export when available, with a legacy export fallback
for deployments that do not expose the newer route. Inventory exports need repository contents
read access. Dependabot alerts require the feature to be available and the current `gh` login
to have access to security alerts (fine-grained tokens need **Dependabot alerts: read**).
See [GitHub's SBOM API](https://docs.github.com/en/rest/dependency-graph/sboms) and
[Dependabot API](https://docs.github.com/en/rest/dependabot/alerts).

Missing permissions, unavailable features, malformed responses, and incomplete pagination
produce an **unavailable/unknown** state, never a zero-vulnerability result. Inventory and
security succeed or fail independently. Last successful snapshots persist across restarts and
remain explicitly labeled as last known when collection fails. Only a complete successful
empty alert response establishes zero open alerts; observations older than two hours are
shown as stale. Collection does not enable GitHub security features or expand token permissions.

## GitHub Actions and test reports

Declare where CI and JUnit evidence lives:

```python
from scryr import CICD, ActionsReportSource, Manifest, TestReportSource, Tests

api = Manifest(
    manifest_id="services/api",
    name="Public API",
    tests=Tests(
        source=TestReportSource(files=["results/junit.xml"], suite="unit")
    ),
    cicd=CICD(
        source=ActionsReportSource(workflow_id=42, branch="main")
    ),
)
```

Report completed observations from CI without hard-coding volatile results into the manifest:

```sh
scryr report tests --manifest api --run-id "$GITHUB_RUN_ID" --observed-at "$RESULTS_COMPLETED_AT"
scryr report actions --manifest api --event-file workflow-run.json
```

## Prometheus-compatible metrics

`PrometheusSource` declares bounded queries against a Prometheus-compatible backend. Grafana installations work when they expose a compatible data source/API. This lets an investigation move directly from a component on the diagram to the production signals that explain its behavior, without treating a dashboard and an architecture diagram as separate worlds.

```python title="index.scry"
from scryr import CredentialRef, Manifest, Metrics, PrometheusSource

api = Manifest(
    name="Public API",
    metrics=Metrics(
        provider=PrometheusSource(
            credentials=CredentialRef(name="production-prometheus"),
            environment="production",
            queries={
                "request_rate": "sum(rate(http_requests_total[5m]))",
                "error_rate": "sum(rate(http_requests_total{status=~'5..'}[5m]))",
            },
            units={"request_rate": "req/s", "error_rate": "errors/s"},
        )
    ),
)
```

Credentials are referenced by name rather than stored in the `.scry` file. Query windows, steps, cache duration, and result size are bounded by the SDK.

```sh
scryr query --list
scryr query request_rate --manifest api
```

Queries execute through Scryr's GraphQL server, allowing credentials to remain server-side.

## PostHog analytics

`PostHogSource` declares analytics queries with start, end, and environment placeholders. Use an explicit provider when the same query name exists in more than one provider.

```sh
scryr query page_views --provider posthog --json
```

## GraphQL API

The diagram reads canonical blocks, nested diagrams, source documents, reports, and observations through Scryr's GraphQL API. Local defaults are:

- UI: `http://127.0.0.1:8000/`
- GraphQL and WebSocket: `http://127.0.0.1:8000/graphql`
- Playground: `http://127.0.0.1:8000/playground`
- Liveness: `http://127.0.0.1:8000/health`
- Readiness: `http://127.0.0.1:8000/ready`

## mise, Docker Compose, and devcontainers

Model a developer environment once as a `Forge`, then export:

```sh
scryr export mise --forge "Developer environment" > mise.toml
scryr export compose --forge "Developer environment" > compose.yaml
scryr export devcontainer --forge "Developer environment" > devcontainer.json
```

The outputs are generated from the same typed declaration that components reference on the diagram.
