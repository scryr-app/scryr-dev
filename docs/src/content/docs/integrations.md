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

## GitHub Actions and test reports

Declare where CI and JUnit evidence lives:

```python
from scryr import ActionsReportSource, CICD, Manifest, TestReportSource, Tests

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

`PrometheusSource` declares bounded queries against a Prometheus-compatible backend. Grafana installations work when they expose a compatible data source/API.

```sh
scryr query --list
scryr query request_latency --manifest api
```

Queries execute through Scryr's GraphQL server, allowing credentials to remain server-side.

## PostHog analytics

`PostHogSource` declares analytics queries with start, end, and environment placeholders. Use an explicit provider when the same query name exists in more than one provider.

```sh
scryr query page_views --provider posthog --json
```

## GraphQL API

The map reads canonical blocks, diagrams, source documents, reports, and observations through Scryr's GraphQL API. Local defaults are:

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

The outputs are generated from the same typed declaration that components reference on the map.
