---
title: Laptop evidence and collectors
description: Declare local repository, checks, metrics, tests, dependencies, and performance evidence in typed Python.
---

Scryr runs alongside your project. Define concrete integrations in `index.scry`;
`scryr serve` schedules collection on the laptop, stores observations locally, and
updates the cards. The browser reads observations. Opening a card never launches
a test, scanner, or benchmark.

Each of `repository`, `checks`, `metrics`, `tests`, `dependencies`, and
`performance` is a typed list. A nonempty list creates one card, with collector
panels in list order. An empty list hides that card. There is no generic provider
wrapper or separate authored collector configuration.

## Repository and source links

Git context works without GitHub. Optional GitHub CLI integrations use the user's
existing `gh` login to read PRs and workflow runs. Remote CI belongs to Repository;
it is labeled with its own branch/commit and never implies a dirty checkout passed.

```python
from scryr import Info, Link, Manifest
from scryr.collectors import GitHubActionsCollector, GitHubPullRequestCollector, GitStatusCollector

api = Manifest(
    manifest_id="services/api",
    name="API",
    repository=[
        GitStatusCollector(directory="."),
        GitHubPullRequestCollector(repository="acme/api"),
        GitHubActionsCollector(repository="acme/api", branch="main", workflow="ci.yml"),
    ],
    info=Info(links=[Link(site_name="Source", http_url="https://github.com/acme/api")]),
)
```

Git reads local refs; there is no implicit `git fetch`. Missing credentials or an
unreachable network keep their own collector status, while local Git continues.

## Local checks and test reports

Concrete checks run the project's actual tools. Tests arrange fresh result files;
report-only collectors watch existing files and never start a runner.

```python
from scryr.collectors import (
    CargoClippyCollector, JUnitReportCollector, LcovCoverageCollector,
    PytestCollector, RuffCheckCollector,
)

api = Manifest(
    manifest_id="services/api",
    name="API",
    checks=[RuffCheckCollector(), CargoClippyCollector(directory="crystal")],
    tests=[
        PytestCollector(id="unit", paths=["tests/unit"]),
        JUnitReportCollector(id="integration", files=["results/integration.xml"], suite="integration"),
        LcovCoverageCollector(files=["coverage/lcov.info"], suite="unit"),
    ],
)
```

`BiomeCheckCollector`, `MiseTaskCollector`, `VitestCollector`, `NextestCollector`,
and `CoberturaCoverageCollector` cover additional project tools/formats. Runners
are manual by default. File reports watch their declared paths. A failed test is
a valid result; an absent or malformed artifact is a collection error.

```sh
scryr collect doctor
scryr collect list
scryr collect run --manifest services/api --section tests --collector unit
scryr collect status
```

## OpenMetrics from one endpoint

The CLI scrapes application/exporter metrics directly. Start with one endpoint:

```python
from scryr.collectors import OpenMetricsCollector

api = Manifest(
    manifest_id="services/api",
    name="API",
    metrics=[OpenMetricsCollector(endpoint="http://127.0.0.1:8080/metrics")],
)
```

No Prometheus server or PostHog account is required. The default view shows bounded
metric discovery. For a focused summary, use typed selections:

```python
from scryr.collectors import CounterRate, GaugeSeries, HistogramPercentile

metrics = [OpenMetricsCollector(
    endpoint="http://127.0.0.1:8080/metrics",
    series=[
        CounterRate(metric="http_requests_total", title="Requests"),
        HistogramPercentile(metric="http_request_duration_seconds", percentile=95),
        GaugeSeries(metric="process_resident_memory_bytes", title="Memory"),
    ],
)]
```

Counter rates require successive observations. Unsupported/missing histograms are
unavailable, not invented percentiles. Scrape success is not an application health
claim. `DockerStatsCollector(containers=["api"], context="default")` optionally
adds resource measurements when a local Docker engine already exists.

## Dependencies: inventory, licenses, vulnerabilities

The card is named **Dependencies**. License evidence and vulnerability evidence
are equally visible alongside package inventory.

```python
from scryr.collectors import (
    GrantLicenseCollector, GrypeScanCollector, LicensePolicy, SbomRef,
    SyftInventoryCollector,
)

api = Manifest(
    manifest_id="services/api",
    name="API",
    dependencies=[
        SyftInventoryCollector(id="packages", directory="."),
        GrantLicenseCollector(
            sbom=SbomRef(collector_id="packages"),
            policy=LicensePolicy(allow=["MIT", "Apache-2.0", "BSD-3-Clause"]),
        ),
        GrypeScanCollector(sbom=SbomRef(collector_id="packages")),
    ],
)
```

Syft supplies packages, established relationships, and detected/declared licenses.
Grant enumerates licenses from that inventory; Scryr evaluates its original SPDX
expressions against the project's typed policy, preserving `AND`, `OR`, and
`WITH` semantics. Unknown or unlisted licenses need review. This is evidence
against a configured policy, not a universal license-compliance guarantee.
Grype matches known vulnerabilities and
records scanner/database provenance. Neither scanner starts a duplicate inventory.

`SbomRef` may include `manifest_id` for another declared manifest. Omitting the
reference works only when exactly one inventory exists in the same Dependencies
section. Invalid references and scanner-to-scanner cycles fail source validation.

Policy changes reevaluate cached inventory. Advisory refreshes rescan even when
lockfiles have not changed. Derived results from an older inventory are marked
outdated. Missing tools, unknown licenses, unsupported ecosystems, and incomplete
scans never become fabricated clean counts.

## Performance benchmarks

Benchmarks run deliberately and retain repeat counts, variability, and machine
context. Resource samples stay in Metrics.

```python
from scryr.collectors import Command, HyperfineBenchmarkCollector

api = Manifest(
    manifest_id="services/api",
    name="API",
    performance=[HyperfineBenchmarkCollector(
        id="startup", command=Command(executable="./bin/api", args=["--version"]),
        warmup=2, runs=10,
    )],
)
```

A comparable baseline is required for regression claims. Absolute results remain
useful without one. Hyperfine defaults to manual execution.

## Schedules, identity, and tool setup

Use stable `manifest_id` values. Collector identity combines that ID, section,
and collector ID. Default IDs are integration kinds, such as `git_status` and
`pytest`; declare unique IDs when repeating an integration in the same section.

```python
from datetime import timedelta
from scryr.collectors import EnvRef, Schedule, ToolRequirement

metrics = [OpenMetricsCollector(
    endpoint="http://localhost:8080/metrics",
    auth=EnvRef(name="METRICS_TOKEN"),
    schedule=Schedule(startup=True, every=timedelta(seconds=30)),
    timeout=timedelta(seconds=10),
)]
```

Only environment variable names go into source and stored configuration. GitHub
reads use tool-managed credentials. `ToolRequirement(version=">=1,<2")` constrains
the fixed integration tool; inspect `doctor` output for effective project paths and
supported versions. Starting Scryr does not automatically install external tools.

All paths are relative to the real entrypoint within the registered project.
Commands use executable/argv, and tool-specific report settings are generated from
typed SDK declarations. Only trusted local source may register execution. Browser
preview, uploaded manifests, and ordinary GraphQL writes do not grant that ability.

## Example card displays

These values are illustrative, not measurements of your project.

| Card | Example |
| --- | --- |
| Repository | `feature/search · 3 modified files`; `PR #42 · review requested`; `GitHub CI main@abc123 · passed` |
| Checks | `Ruff passed · 0 findings`; `Clippy failed · 2 diagnostics`; last checkout revision |
| Metrics | `Requests 42/s`; `p95 83 ms`; `Memory 118 MiB`; scrape age and target |
| Tests | `128 passed · 2 failed · 4 skipped`; `84% line coverage`; suite/commit provenance |
| Dependencies | `142 packages`; `136 licenses identified · 6 unknown`; `8 policy reviews`; `8 vulnerabilities across 5 packages` |
| Performance | `123 ms ± 4 ms · 10 runs`; compatible baseline comparison or `No comparable baseline` |

A declared collector is visible before first success, with waiting, running,
missing-tool, login, error, or stale status. Existing results retain their true
age after errors. Removing a collector stops its work and hides its current panel;
retained history remains available until retention expires.

## GraphQL API

The server exposes typed evidence, collector status, and history alongside blocks
and source documents. Ingestion updates observations without regenerating the
architecture. Local UI and GraphQL defaults are `http://127.0.0.1:8000/` and
`http://127.0.0.1:8000/graphql`; readiness is `/ready` and liveness is `/health`.

This is a breaking evidence contract. Old wrappers, provider queries, and report
commands are removed. An incompatible database must fail clearly; follow explicit
export/reset/reseed instructions instead of silently deleting stored data.

## mise, Docker Compose, and devcontainers

Model tools and tasks using `Forge`, then generate project artifacts:

```sh
scryr export mise --forge "Developer environment" > mise.toml
scryr export compose --forge "Developer environment" > compose.yaml
scryr export devcontainer --forge "Developer environment" > devcontainer.json
```

`MiseTaskCollector` may reference a Forge task as a local check. Its task result
is not parsed as test coverage or a vulnerability scan.
