# Scryr Python SDK

Define architecture with `Manifest`, `Diagram`, `Info`, and `Forge`, and attach
laptop evidence using typed integration classes. The SDK never executes a
collector while importing, constructing models, serializing, or previewing them.
Native `scryr serve` owns execution after a local project is registered.

## Evidence sections

| Manifest field | Concrete integrations | Card |
| --- | --- | --- |
| `repository` | `GitStatusCollector`, `GitHubPullRequestCollector`, `GitHubActionsCollector` | Repository: local Git context and optional remote PR/CI evidence |
| `checks` | `RuffCheckCollector`, `BiomeCheckCollector`, `CargoClippyCollector`, `MiseTaskCollector` | Checks: local verification |
| `metrics` | `OpenMetricsCollector`, `DockerStatsCollector` | Metrics: measured application/resource samples |
| `tests` | `PytestCollector`, `VitestCollector`, `NextestCollector`, `JUnitReportCollector`, `LcovCoverageCollector`, `CoberturaCoverageCollector` | Tests: suites and measured coverage |
| `dependencies` | `SyftInventoryCollector`, `GrantLicenseCollector`, `GrypeScanCollector` | Dependencies: inventory, licenses, vulnerabilities |
| `performance` | `HyperfineBenchmarkCollector` | Performance: deliberate benchmarks |

Import classes from `scryr.collectors` or their integration modules, such as
`scryr.collectors.syft` and `scryr.collectors.openmetrics`. There are no generic
provider wrappers, manifest-wide collectors, static metric fields, or legacy
GitHub/CI section aliases. Wrong-section collectors and unknown options fail
Pydantic validation. Any populated evidence section requires `manifest_id`.
Collector IDs default to their wire kind and must be unique within the section.

```python
from datetime import timedelta
from scryr import Diagram, Manifest
from scryr.collectors import (
    GitStatusCollector,
    GrantLicenseCollector,
    GrypeScanCollector,
    LicensePolicy,
    OpenMetricsCollector,
    PytestCollector,
    SbomRef,
    Schedule,
    SyftInventoryCollector,
)

api = Manifest(
    manifest_id="services/api",
    name="API",
    repository=[GitStatusCollector()],
    metrics=[OpenMetricsCollector(endpoint="http://127.0.0.1:8080/metrics")],
    tests=[PytestCollector(id="unit", paths=["tests/unit"], timeout=timedelta(minutes=5))],
    dependencies=[
        SyftInventoryCollector(id="packages", schedule=Schedule(startup=True, watch=["uv.lock"])),
        GrantLicenseCollector(
            sbom=SbomRef(collector_id="packages"),
            policy=LicensePolicy(allow=["MIT", "Apache-2.0"]),
        ),
        GrypeScanCollector(sbom=SbomRef(collector_id="packages")),
    ],
)
diagram = Diagram(name="Local API", manifests=[api])
```

A license allowlist is a project policy, not a universal compliance claim. Unknown
or unlisted licenses remain review findings. Syft alone provides detected license
evidence; Grant supplies license inventory for Scryr's typed SPDX policy decisions;
Grype adds vulnerability evidence. Both
reference the same inventory. An omitted `sbom` is valid only when exactly one
Syft inventory is declared. Cross-manifest references additionally specify
`manifest_id`; the source envelope validates their existence and target type.
Only Syft produces inventory, so scanner-to-scanner reference cycles are invalid.

## Scheduling and tools

`Schedule(startup=True, every=timedelta(seconds=30), watch=["src/**"])` combines
triggers. `manual=True` is the common default; Git polls every 15 seconds, GitHub
reads every 180 seconds, and metric scrapes every 30 seconds. Test/check/benchmark
runners remain manual unless explicitly scheduled. File-report collectors watch
their declared files without running tests. Grant follows inventory changes;
Grype follows inventory changes and refreshes daily for advisory changes.

`timeout`, `freshness`, `Schedule.every`, and `Schedule.debounce` accept Python
`timedelta` and serialize as seconds. Invalid durations and empty schedules fail.
Paths resolve relative to the real `index.scry`, within the registered project.
They cannot escape lexically; the CLI must also enforce symlink boundaries.

`ToolRequirement(version=">=1,<2")` constrains an integration's fixed tool; it
cannot replace Syft with another scanner. No external tool installation happens
on model construction or server startup. `scryr collect doctor` checks the active
project environment and explains missing/incompatible tools.

Use `EnvRef(name="API_TOKEN")` in `env` mappings or OpenMetrics `auth`; only the
variable name is serialized. Secrets remain in the laptop's environment or a
tool's managed login. `Command(executable="python", args=["--version"], cwd=".")`
provides argv for Hyperfine; shell interpolation is not part of that interface.

## OpenMetrics

The CLI scrapes the application/exporter endpoint directly. No Prometheus server,
query language, PostHog account, or separate provider configuration is required.
The default view discovers bounded metric names, types, units, and samples.
Selections use `CounterRate`, `GaugeSeries`, or `HistogramPercentile`:

```python
from scryr.collectors import CounterRate, GaugeSeries, HistogramPercentile

metrics = [
    OpenMetricsCollector(
        endpoint="http://localhost:8080/metrics",
        series=[
            CounterRate(metric="http_requests_total", title="Requests"),
            HistogramPercentile(metric="http_request_duration_seconds", percentile=95),
            GaugeSeries(metric="process_resident_memory_bytes", title="Memory"),
        ],
    )
]
```

A first counter sample cannot establish a rate. Missing series, unsupported
histograms, stale scrapes, or missing instruments are unavailable evidence,
not zero measurements or a health claim.

## Serialization and source tools

Architecture retains its established field aliases such as `manifestId` and
`Info.ownerTeam`. Collector configuration deliberately uses snake_case throughout,
including `upstream_changed`, `collector_id`, and `max_series`. Each concrete class
has a literal `kind` for wire discrimination; users construct typed classes.
`Manifest.model_json_schema(mode="serialization")` includes their complete schema.
`run_manifest_file` validates the full source envelope before emitting values or
metadata, including collectors on diagram-only manifests and imported components.

The prior section wrappers, Python action networking clients, provider query
models, and old report APIs are removed. Update source declarations instead of
relying on compatibility aliases. The new database version is explicit; do not
delete a database implicitly to make an old installation load.
