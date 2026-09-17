# CLI-focused cards implementation plan

Planning baseline: `91167cd` on `cli-focused-cards`. Revised to use new SDK section
names and integration-specific collectors, without backward compatibility. This
records the approved design; the implementation is on `cli-focused-cards`. Upstream tool
and OpenMetrics documentation was checked on September 17, 2026.

## Recommended direction

Make `scryr serve` the local collection host as well as the local UI/API server.
It reads typed Python declarations from `index.scry`, schedules work on the laptop,
normalizes integration-specific results, and persists observations. The cards
read those observations without executing commands in the browser.

The manifest SDK exposes six evidence sections: `repository`, `checks`, `metrics`,
`tests`, `dependencies`, and `performance`. Each is a typed list of concrete
collectors accepted by that section. There is no manifest-wide `collectors` list,
no public `Collector(source=...)` wrapper, and no separate section configuration
object such as `Dependencies(...)`. Commands, schedules, tool requirements,
license policies, metric selections, report paths, and thresholds all use typed
SDK models in `index.scry`.

Remove obsolete APIs, files, names, and execution paths as part of implementation.
Do not retain aliases, old serialized fields, dual readers, static summary
fallbacks, or the existing provider architecture to preserve compatibility.
Regenerate samples and bundled artifacts against the new SDK. Reuse sound internal
parsing, persistence, authorization, and lifecycle primitives where they fit.

Require external tools only for enabled collectors. Missing tools produce clear
card states while the map and other collectors keep working. The default result
destination is local GraphQL/SQLite, with no Scryr cloud account required. Secrets,
installed binaries, caches, and history are machine state; users do not need a
second authored Scryr configuration file. Remote result sync remains a later,
explicitly configured feature.

## SDK sections and card names

| SDK field and card | Typed collector examples | Purpose and default behavior |
| --- | --- | --- |
| `repository` → Repository | `GitStatusCollector`, `GitHubPullRequestCollector`, `GitHubActionsCollector` | Local branch/HEAD/dirty state and optional GitHub PR/review/CI context. Git every 15 seconds; remote reads every 2–5 minutes. Upstream divergence uses the last fetched refs; no implicit fetch. |
| `checks` → Checks | `RuffCheckCollector`, `BiomeCheckCollector`, `CargoClippyCollector`, `MiseTaskCollector` | Local lint, formatting, type, build, or other explicitly named verification tasks. Manual by default; input-change triggers are opt-in. Remote CI belongs to Repository. |
| `metrics` → Metrics | `OpenMetricsCollector`, optional `DockerStatsCollector` | Current instrumented application/resource measurements collected by the CLI. OpenMetrics scrapes every 30 seconds by default; no Prometheus server or PostHog account required. |
| `tests` → Tests | `PytestCollector`, `VitestCollector`, `NextestCollector`, `JUnitReportCollector`, `LcovCoverageCollector`, `CoberturaCoverageCollector` | Local suite outcomes and measured coverage. Runner collectors default to manual; report collectors watch declared artifacts. |
| `dependencies` → Dependencies | `SyftInventoryCollector`, `GrypeScanCollector`, `GrantLicenseCollector` | Package inventory and relationships, license evidence/policy findings, and vulnerability findings. Inventory follows input changes; license and security collectors consume that inventory. |
| `performance` → Performance | `HyperfineBenchmarkCollector`, later `K6Collector` | Deliberate benchmarks, variability, baselines, and machine/runtime context. Manual by default, with bounded runs and duration. |

Rename the actual SDK fields, constructor arguments, Python exports, generated
metadata, serialized contracts, GraphQL fields, TypeScript types, card IDs, query
field references, sample manifests, and documentation. `github` becomes
`repository`; `cicd` becomes `checks`, with remote workflow evidence explicitly
moved to `repository`. The visible labels are exactly Repository, Checks, Metrics,
Tests, Dependencies, and Performance.

Integration names remain specific: `GitHubPullRequestCollector` and
`GitHubActionsCollector` are the concrete equivalents of a pull-request collector
and a CI collector. They integrate with GitHub through `gh`; they are not generic
provider-switching models. Future GitLab support would have its own classes.

## Remove the obsolete implementation paths

| Current area | Planned replacement or removal |
| --- | --- |
| `manifest/scryr/src/scryr/manifest.py` and `__init__.py` | Replace old card section models and flattened aliases with typed per-section collector unions. Remove `Github`, `CICD`, old `Metrics`/`Tests`/`Dependencies`/`Performance` evidence wrappers, `TestReportSource`, and `ActionsReportSource` once their callers use concrete collectors. Preserve unrelated architecture fields and useful Forge models. |
| `manifest/scryr/src/scryr/metrics_source.py` | Remove `PrometheusSource`, `PostHogSource`, their query dictionaries, and provider-specific credential/config machinery. Replace with the OpenMetrics collector SDK module. Remove the old `analytics` field and HogQL integration from this design. |
| `crystal/crystal-server/src/runtime_metrics.rs` | Remove server-side provider fetching, `SCRYR_METRICS_CONNECTIONS_FILE`, module/schema initialization in `server.rs`, and diagram-open collection. CLI collectors own metric acquisition. |
| `crystal/crystal-cli/src/commands/query.rs` and its argument definitions | Remove `QueryArgs` and old command dispatch/help; use `scryr collect` for collection/inspection and GraphQL for stored observations. Remove old section lookups from report configuration/import commands. |
| `crystal/crystal-server/src/roots.rs` and schema wiring | Replace the GraphQL fields `manifestQuery`, `diagramMetrics`, and special Actions-only projection with typed evidence reads for the six sections. Refactor useful Actions parsing into the GitHub collector. |
| `map/src/graphql/useDiagramMetrics.ts`, its tests, and consumers in `useBlocksData.ts` | Remove browser-triggered provider fetching. Read projected collector observations through the new operations. |
| `map/src/cards/blockCardData.ts`, `ReportCard.tsx`, and `map/src/block/defaultCards.tsx` | Replace old static/provider/report branches with section-specific card view models and concrete collector result renderers. Remove old card identifiers and fallback rendering. |
| `map/src/cards/GithubCard.tsx`, `CICDCard.tsx`, and `RuntimeMetricsCard.tsx` | Replace with `RepositoryCard.tsx`, `ChecksCard.tsx`, and a persisted-evidence Metrics card. Remove provider-specific rendering, old block fields such as `cicdTool`, and corresponding GraphQL operation assumptions. |
| Existing report/history contracts | Replace obsolete report shapes and redundant ingestion paths with one canonical observation pipeline. Keep no v1 reader or legacy evidence mode solely for compatibility. |

Audit SDK helpers, CLI reporting commands, Actions clients, tests, samples, and
docs for orphaned paths. Delete them when the new pipeline replaces their role;
rename/refactor useful implementation code into the new structure. No parallel
old/new reporting system should remain at completion.

Audit `manifest/scryr/src/scryr/actions.py` and `action_client.py` for network/reporting
entry points superseded by native collectors, and `map/src/pyodide/githubShim.py`
for obsolete browser imports. Remove unnecessary entry points and shims. Useful
run/event parsing can move into the canonical GitHub integration without retaining
the old public API. Regenerate source snapshots and sample seed data for the new
schema rather than translating old stored diagrams.

Use a new database schema version and reject an incompatible database clearly.
Implementation can document an explicit export/reset/reseed path instead of
building old-data compatibility. Do not silently delete a user's database; no
reset or data deletion is part of this planning change.

## Integration-specific SDK design

Place public classes in integration modules, for example
`scryr.collectors.syft`, `.grype`, `.grant`, `.github`, `.openmetrics`, `.pytest`,
and `.hyperfine`, with convenient exports from `scryr.collectors`. Each class owns
its integration's options, required tools, output parsing contract, typed results,
and defaults. Users never select an adapter through a free-form string/dictionary.

Shared value types such as `Schedule`, `Command`, `ToolRequirement`, `SbomRef`,
`EnvRef`, and `LicensePolicy` are allowed. An internal base model can share validated
lifecycle fields, but it is not a public catch-all collector. Commands and arbitrary
argv overrides must not erase the contract of a named integration: e.g. a
`GrypeScanCollector` cannot be configured to execute a different scanner. A
`MiseTaskCollector` specifically runs a declared mise/Forge task and reports its
check outcome; it does not impersonate a test or security parser.

Each section has a closed typed union of supported collectors, with a literal
integration discriminator used only in serialization. Python typing and Pydantic
validation reject a `GrypeScanCollector` placed in `repository`, a GitHub Actions
collector placed in `checks`, unknown fields, and invalid integration options.
For example, the initial `dependencies` field is typed as
`list[SyftInventoryCollector | GrantLicenseCollector | GrypeScanCollector]`;
the repository union is independently restricted to its own integration classes.

Illustrative `index.scry` using the proposed SDK:

```python
from datetime import timedelta

from scryr import Manifest
from scryr.collectors import (
    CargoClippyCollector, GitHubActionsCollector, GitHubPullRequestCollector,
    GitStatusCollector, GrantLicenseCollector, GrypeScanCollector,
    HyperfineBenchmarkCollector, LicensePolicy, OpenMetricsCollector,
    PytestCollector, RuffCheckCollector, SbomRef, Schedule, SyftInventoryCollector,
    Command,
)

api = Manifest(
    manifest_id="services/api",
    name="API",
    repository=[
        GitStatusCollector(id="git", directory="."),
        GitHubPullRequestCollector(id="prs", repository="example/api"),
        GitHubActionsCollector(
            id="ci", repository="example/api", branch="main", workflow="ci.yml",
        ),
    ],
    checks=[
        RuffCheckCollector(id="lint", directory="."),
        CargoClippyCollector(id="rust-lint", directory="crystal"),
    ],
    metrics=[
        OpenMetricsCollector(id="api-metrics", endpoint="http://127.0.0.1:8080/metrics"),
    ],
    tests=[
        PytestCollector(
            id="unit", directory=".", paths=["tests/unit"],
            schedule=Schedule(manual=True), timeout=timedelta(minutes=5),
        ),
    ],
    dependencies=[
        SyftInventoryCollector(
            id="inventory", directory=".",
            schedule=Schedule(startup=True, watch=["uv.lock", "pyproject.toml"]),
        ),
        GrantLicenseCollector(
            id="licenses", sbom=SbomRef(collector_id="inventory"),
            policy=LicensePolicy(allow=["MIT", "Apache-2.0", "BSD-3-Clause"]),
        ),
        GrypeScanCollector(id="vulnerabilities", sbom=SbomRef(collector_id="inventory")),
    ],
    performance=[
        HyperfineBenchmarkCollector(
            id="startup-time",
            command=Command(executable="./bin/api", args=["--version"], cwd="."),
            warmup=2, runs=10, timeout=timedelta(minutes=2),
        ),
    ],
)
```

This example's license allowlist is a project policy example. Unknown or unlisted
licenses default to review. Users can declare stricter, typed policy decisions;
the card reports those decisions and their evidence.

Required semantics:

- Section fields default to empty typed lists. Empty section means no card; a
  nonempty section means one card with one or more collector result groups. Do not
  add a `collectors` field at either the manifest or section-wrapper level.
- Require `manifest_id` when any evidence section is configured. Collector identity
  is `(manifest_id, section, id)`; omitted IDs use documented integration defaults.
  Multiple collectors with the same default ID require explicit unique IDs. List
  order controls display order, not identity or execution dependencies.
- `SbomRef` is a typed reference to an inventory artifact, not a file-path string.
  It resolves inside the same manifest's Dependencies section by default; a
  cross-manifest reference names the manifest explicitly. Detect missing references,
  ambiguous inventory, invalid output types, and cycles before execution. A scanner
  may omit `sbom` only when exactly one declared inventory can supply it.
- Schedules, timeouts, policy, credentials, thresholds, and tool constraints have
  typed SDK models and generated completion metadata. Models forbid unknown fields.
  Use documented integration defaults, with explicit overrides when needed.
- Grant runs after inventory or license-policy changes; Grype runs after inventory
  changes and on its daily advisory refresh, even when the lockfile is unchanged.
  Neither silently launches its own duplicate inventory collector.
- Each integration chooses structured output itself. `PytestCollector` arranges a
  fresh JUnit artifact; `NextestCollector` generates any needed report settings;
  file-only JUnit/coverage collectors ingest existing artifacts without running
  tests. Tool-specific temporary config comes from typed Python declarations.
- The runner resolves tools in the declared project environment. Reuse Forge and
  its tool-version declarations where useful; defaults never silently switch to a
  different project's interpreter or toolchain. Expose tested version constraints
  and show effective executable paths in `scryr collect doctor`.
- Resolve paths from the real entrypoint directory within the registered project
  root, not the shell cwd or editor staging tree. Resolve symlinks and enforce
  declared boundaries. Outputs use fresh per-run directories; typed output refs
  expand into argv elements without shell interpolation.
- SDK constructors, serialization, exports, validation, and browser preview only
  declare work. `.scry` still executes Python, but collector construction never
  starts tools, installs dependencies, or performs network calls.

## Metrics: one CLI OpenMetrics path

The default authoring experience is one collector with one endpoint:

```python
metrics=[OpenMetricsCollector(endpoint="http://127.0.0.1:8080/metrics")]
```

`scryr serve` performs the HTTP scrape itself using its Rust HTTP client. The user
needs no Prometheus server, PromQL/HogQL query, provider registration, named server
connection, or separate metrics config. The collector negotiates and validates
supported [OpenMetrics and Prometheus exposition formats](https://prometheus.io/docs/instrumenting/exposition_formats/),
then records bounded typed samples in Scryr. Scrape success is acquisition status;
it must not be mislabeled as proof that the application is healthy.

Use endpoint defaults for a useful bounded discovery view: show metric names,
help text, declared types/units, latest samples, and recent trends. Provide typed
selection when a project wants a concise application summary:

```python
from scryr.collectors.openmetrics import CounterRate, GaugeSeries, HistogramPercentile

metrics=[
    OpenMetricsCollector(
        endpoint="http://127.0.0.1:8080/metrics",
        series=[
            CounterRate(metric="http_requests_total", title="Requests"),
            HistogramPercentile(
                metric="http_request_duration_seconds", percentile=95, title="Latency p95",
            ),
            GaugeSeries(metric="process_resident_memory_bytes", title="Memory"),
        ],
    ),
]
```

These are typed metric selections, not embedded query languages. Calculate counter
rates and histogram percentiles over collected windows, with per-series reset
handling and explicit label aggregation. The examples select default aggregation
across matching series on this one target; preserve a bounded label drilldown.
Do not average precomputed quantiles or combine unrelated targets. Missing
histogram buckets, incompatible units, partial data, or a first counter sample
show unavailable/collecting status. Define supported format versions and histogram
features; reject unsupported encodings explicitly rather than inventing values.

Bound response bytes, samples, labels, series cardinality, scrape duration, and
retention. Expose typed environment-based auth only when an endpoint needs it.
Default local targets keep laptop setup small; remote endpoints are explicit and
retain existing credential/URL protections.

If someone already uses Prometheus, prefer scraping the application/exporter
endpoint directly. An explicitly selected [Prometheus federation endpoint](https://prometheus.io/docs/prometheus/latest/federation/)
can supply exposed samples; the server's own `/metrics` endpoint describes that
server, not every monitored application. There is no retained Prometheus query API
integration. Remove built-in PostHog/HogQL acquisition entirely: PostHog event
analytics are not automatically OpenMetrics. A separately configured exporter can
expose chosen aggregates, which the same collector can scrape. Building a PostHog
exporter is outside the initial implementation.

## Dependencies: inventory, licenses, and vulnerabilities

Keep the SDK name `dependencies` and card label Dependencies. Give three concerns
first-class typed results and card content:

- **Inventory:** packages, versions, ecosystems, source files, and dependency edges
  actually established by Syft. Keep package edges separate from architectural
  `Manifest.connections`; do not manufacture direct/transitive relationships.
- **Licenses:** display Syft's detected/declared license evidence even without a
  policy collector. `GrantLicenseCollector` adds results against the typed project
  policy. Preserve SPDX expressions, AND/OR structure, exceptions, evidence paths,
  unknown/unlicensed cases, and package associations. Distinguish observed license
  evidence from policy decisions. A package with no license evidence needs review.
- **Vulnerabilities:** `GrypeScanCollector` reports package/advisory/fix details,
  severity and its source, database age, and scan completeness. Missing scanners
  or incomplete ecosystem coverage do not mean zero vulnerabilities.

[Grant](https://oss.anchore.com/docs/guides/license/) can inspect SBOM licenses and
apply policies. Compile typed policy declarations into private generated Grant
configuration when needed; users do not author an additional Grant YAML file.
Validate expression semantics against supported Grant versions, preserve unknown
or unsupported expressions for review, and store policy revision with results.
Changing policy reruns evaluation against existing inventory without rescanning
unchanged project files. Neither scanner output nor a green policy result is a
universal license-compliance claim; show the configured policy and actual evidence.

Match license/security evidence to the same inventory fingerprint and package
identity. On an inventory update, show previous derived results as outdated until
recomputed. Deduplicate advisory aliases per affected package while retaining
scanner attribution. Do not combine unrelated scan roots or count the same package
repeatedly simply because several collectors report it.

## How collector declarations shape the cards

One populated SDK section creates one card. Every concrete collector has a typed
result renderer registered under that card's section. The UI builds the view from
configured collectors plus their current observations; it does not infer a card
from arbitrary JSON or a non-null static summary field. Card names are stable,
while collector rows/panels follow the configured list order.

The following displays use illustrative results, not measurements of this repo:

| Section configuration | Example card display | How it responds to change |
| --- | --- | --- |
| `repository=[GitStatusCollector(...), GitHubPullRequestCollector(...), GitHubActionsCollector(...)]` | **Repository** — `feature/search · 3 modified files`; `PR #42 · review requested`; `GitHub Actions · 2 passed / 1 running · main@a1b2c3`. | Each collector has its own timestamp/status. CI is labeled by branch/commit so a passing main run never implies the dirty local checkout passed. Removing the Actions collector removes the CI panel only. |
| `checks=[RuffCheckCollector(...), CargoClippyCollector(...)]` | **Checks** — `Ruff: passed · 0 findings`; `Clippy: failed · 2 diagnostics`; `Run checks`. | Run the selected declared local checks. Group summaries only for a matching checkout/input revision; a code edit makes previous results outdated. CI never appears here. |
| `metrics=[OpenMetricsCollector(...)]` | **Metrics** — `Requests 42/s`; `Latency p95 83 ms`; `Memory 118 MiB`; `Scraped 8s ago`. | First scrape shows `Collecting rate samples`; later samples drive sparklines. An unreachable endpoint keeps last successful values and marks them stale, with the scrape error. |
| `tests=[PytestCollector(...), LcovCoverageCollector(...)]` | **Tests** — `Unit: 128 passed · 2 failed · 4 skipped · 6.2s`; `Line coverage: 84%`; `Local · dirty checkout`. | Show the suite failure details; distinguish test failures from a broken runner. Coverage is shown with matching provenance, otherwise separately marked older/unassociated. |
| `dependencies=[SyftInventoryCollector(...), GrantLicenseCollector(...), GrypeScanCollector(...)]` | **Dependencies** — `142 packages`; `Licenses: 136 identified · 6 unknown`; `Policy: 8 need review`; `Vulnerabilities: 8 findings across 5 packages`. | Expand Inventory / Licenses / Vulnerabilities within the same card. Policy review can include 6 unknown and 2 unlisted licenses. Remove Grype to omit vulnerability results; keep inventory and licenses. A configured but missing Grype shows `Grype required`, never `0 vulnerabilities`. |
| `performance=[HyperfineBenchmarkCollector(...)]` | **Performance** — `startup-time: mean 123 ms ± 4 ms · 10 runs`; `5% slower than selected baseline`; `Run benchmark`. | Show the selected compatible machine/input baseline. With no comparable baseline, show absolute results and `No comparable baseline`, not a regression claim. |

For Dependencies, detail panels join to package records and allow license expression,
policy state, severity, ecosystem, and source-path filters. A basic
`dependencies=[SyftInventoryCollector()]` still shows inventory and license
coverage; it makes no claim about vulnerability scanning or license policy checks.

Every configured collector is visible before its first result: Waiting, Running,
Missing tool, Needs login, Failed to collect, or Stale. Valid findings such as failed
tests and policy review remain result states, separate from collector failures.
Removing a collector hides its current panel and stops its work; retained history
is available explicitly until retention expires. `repository=[]` hides Repository.
Opening a card only reads observations. Local Run/Pause controls use registered
collector IDs and the execution boundary described below.

## Open source tools and scope

Each named class uses the integration's structured interface and a tested output
schema. Tool support is independent of backward compatibility with the old Scryr
SDK. Pin tested tool versions in shipped examples.

| Tool | Concrete collector and interface | Scope |
| --- | --- | --- |
| Git + [GitHub CLI](https://cli.github.com/manual/gh_pr_status) | `GitStatusCollector` uses porcelain output; `GitHubPullRequestCollector` uses PR JSON; `GitHubActionsCollector` uses [`gh run list --json`](https://cli.github.com/manual/gh_run_list). | Initial Repository integrations. Remote reads need network/auth and bounded pagination/rate limits. |
| [Syft](https://oss.anchore.com/docs/reference/syft/cli/) | `SyftInventoryCollector`: `syft scan dir:. -o syft-json`, with CycloneDX export. | Initial inventory and detected license evidence. Explicit directory scope avoids accidentally pulling an image. |
| [Grype](https://oss.anchore.com/docs/reference/grype/cli/) | `GrypeScanCollector`: `grype sbom:<artifact> -o json`. | Initial vulnerability findings against the declared inventory; record advisory database freshness. |
| [Grant](https://oss.anchore.com/docs/reference/grant/cli/) | `GrantLicenseCollector`: structured license/policy output from the same SBOM. | Initial optional license-policy integration. Tool exit status alone cannot distinguish every finding from an operational error; validate report completeness. |
| [pytest](https://docs.pytest.org/en/stable/how-to/output.html), [Vitest](https://vitest.dev/guide/reporters.html), [cargo-nextest](https://nexte.st/docs/machine-readable/junit/) | Separate `PytestCollector`, `VitestCollector`, `NextestCollector`; each manages its report generation. | Initial Python slice, then JS/Rust runner integrations. Reuse strict JUnit/coverage parsers internally; report-only classes remain concrete format integrations. |
| [Hyperfine](https://github.com/sharkdp/hyperfine/blob/master/README.md) | `HyperfineBenchmarkCollector` uses bounded runs, warmups, JSON output, and shell-free execution where supported. | Initial benchmark integration. |
| [mise](https://mise.jdx.dev/cli/exec.html) | `MiseTaskCollector` for checks; Forge-backed optional tool provisioning. | Reuse existing project tasks without inventing another task language. Generated private config comes from SDK declarations. |
| [Docker CLI](https://docs.docker.com/reference/cli/docker/container/stats/) | `DockerStatsCollector` uses selected targets and JSON stats. | Optional resource metrics. Explicit local engine/context; no Docker requirement for normal Scryr use. |
| [OSV-Scanner](https://google.github.io/osv-scanner/usage/), [Trivy](https://trivy.dev/docs/latest/target/filesystem/) | Future `OsvScanCollector`, `TrivyScanCollector`. | Alternative concrete Dependencies integrations after the initial pipeline. Declare network/database behavior and scanning scope explicitly. |
| [k6](https://grafana.com/docs/k6/latest/results-output/end-of-test/custom-summary/) | Future `K6Collector` uses bounded structured summaries. | Opt-in Performance integration; manual default and explicit targets. |
| [Gitleaks](https://github.com/gitleaks/gitleaks), [act](https://github.com/nektos/act) | Future `GitleaksCheckCollector`, `ActCheckCollector`. | Optional local Checks integrations; redact secret content and label local workflow execution as local. |

## Runtime and data flow

```mermaid
flowchart LR
    S[Typed index.scry] --> V[Validate and publish architecture]
    V --> P[Register local collector plan]
    P --> R[CLI scheduler and process runner]
    R --> A[Typed collector results]
    A --> I[Validated report ingestion]
    I --> D[(Local SQLite)]
    D --> G[GraphQL latest results and history]
    G --> C[Six cards]
```

The CLI owns process execution and scheduling; core owns canonical models,
parsers, persistence, and projections; server owns authorization and GraphQL.
Reuse the same report ingestion service from GraphQL and an in-process local
call where useful. Do not create a second set of CLI-only transport models or
upload whole diagrams after every sample.

`serve` should start collection after successful validation and publication,
whether or not `--watch` is set. `--watch` continues to mean source reload;
collector file triggers are independent. `--server-only` never registers or runs
collectors. A proposed `--no-collect` flag offers an operational pause without
requiring a second configuration file.

On source reload/local editor save, use the existing workspace gate and revision
checks to atomically replace the registered plan after successful publication.
Keep unchanged jobs across unrelated source edits. Compare each job's effective
collector definition and upstream bindings, not just the global plan revision;
list reordering must not invalidate work. Cancel removed/changed jobs and retain
late results only as history when their collector definition is no longer active.
Track input fingerprints separately: a result for changed project inputs is
outdated, and inputs changing during a run must be marked rather than attributed
to one clean snapshot. An invalid manifest keeps the prior diagram and last valid
plan with a visible reload error and pause control. Do not hold the workspace gate
while long-running child processes execute.

Scheduler requirements:

- One active collector owner per canonical workspace/database identity, with a
  lease/lock so multiple `serve` processes cannot duplicate expensive work.
  Worktrees have distinct workspace IDs and local state.
- Default at most two ordinary jobs and one heavy job, with an exclusive benchmark
  lane that pauses conflicting heavy work. Skip overlapping ticks, coalesce input
  changes, debounce file events, exclude generated output directories, and apply
  per-repository network request limits plus retry backoff/jitter.
- Use monotonic intervals, bounded timeouts, capped output and file sizes, closed
  stdin, and process-group cancellation. Ctrl+C stops children. Resume after laptop
  sleep with one due run, not a replay of every missed interval.
- Classify exit codes per adapter. Failed tests, vulnerability findings, and
  benchmark threshold failures can be valid observations. Spawn failures, malformed
  output, authentication failures, and timeouts are collector failures.
- Fresh run directories prevent old JUnit or JSON files from being reported as a
  new success. Report watchers wait for stable/atomic writes and deduplicate by
  content plus declared scope; a missing artifact never means zero failures.
- Recompute inventory on relevant input change. Rescan vulnerabilities when the
  advisory database changes or the declared interval expires, even with an
  unchanged lockfile. Persist database version/age and incomplete scan coverage.
- Network failures retain the last successful result with stale/offline status.
  Persist completed results in a bounded local spool before delivery, retry with
  the same run identity, and drain on restart. Expired/oversized entries receive a
  visible diagnostic. Local ingestion is the first release; remote sync is later.

## Observation contract, storage, and card reads

Define one canonical typed observation contract around the new sections and
concrete integration results. Replace old report/dependency/provider payloads;
there is no legacy report mode. Keep version negotiation for future evolution,
but do not carry v1 readers or fake GitHub alert IDs for local scanner findings.

Core result variants cover Git state, GitHub PRs/workflows, local checks, suites,
coverage, inventory, license evidence/policy evaluations, vulnerabilities, metric
samples, and benchmarks. Their public GraphQL types reflect the integration and
section; shared internal value types avoid duplicate package/metric/run models.
The CLI, report importer, and server all use this same contract.

Each observation records `manifestId`, section, collector ID, integration/schema
version, workspace ID, environment (default `local`), scope, global plan revision,
effective collector revision, local execution ID/attempt, immutable observation ID,
start/completion/source/receipt timestamps, tool version, input fingerprint, and
branch/commit/dirty state when known. Keep opaque workspace IDs
instead of personal absolute paths in transferable data. Derived evidence records
its upstream artifact hash and policy revision. GitHub workflow observations retain
their remote branch/commit rather than inheriting the laptop's current commit.

Inventory identifies packages by ecosystem/name/version/purl plus source path.
Licenses retain detected/declared expressions, evidence, completeness, policy rule,
and review outcome. Vulnerabilities retain advisory aliases, severity source,
fixed versions, affected package refs, scanner/database versions, and coverage.
Store missing/unknown findings explicitly; never translate them into successful
zero counts. Counters distinguish packages, unique advisory/package findings,
and policy decisions rather than combining incompatible totals.

Keep lifecycle status separate from successful evidence: disabled, waiting,
running, missing/incompatible tool, missing credentials, error, cancelled.
Fresh/stale/outdated comes from a typed freshness policy, input revision, artifact
revision, and elapsed time. There is no single seven-day threshold for every card.

Use indexed latest-per-(organization, manifest, section, workspace, environment,
collector, kind, scope) projections and bounded history. Write observations and
advance matching projections transactionally; older retries never displace newer
results. Delivery of the same immutable observation ID is idempotent; conflicting
content for that observation is rejected. Separate GitHub's provider run ID/attempt
from observation identity: queued, running, and completed snapshots of the same
workflow are different observations. Deduplicate unchanged provider snapshots using
provider identity, source update/event identity, and normalized payload; apply
explicit transition/tie rules so old snapshots cannot regress a terminal state.
Different real local executions stay distinct even when their summaries match.
Unchanged polling can update a heartbeat instead of adding duplicate snapshots;
metric observations keep meaningful sample times.

Store bounded summaries through GraphQL/database; paginate package, license, and
vulnerability details. Large SBOMs need content-addressed artifacts or chunked
ingestion within configured limits. Raw artifacts live in ignored local cache
with retention. Proposed defaults remain 24 hours of raw metrics with downsampling,
30 days of run summaries, and configurable artifact quotas. Retain the latest
successful result with its real age.

Batch read projections for the six cards to avoid per-block history scans.
Observation ingestion updates cards without re-uploading architecture or changing
`index.scry`. There is no static fallback evidence path. Different tools, suites,
environments, workspaces, and input revisions are never merged blindly. Cross-tool
joins require matching package/inventory or suite/coverage provenance. List order
is presentation order; integration and collector IDs select typed result groups.

Use generated typed GraphQL operations for latest results, history, and collector
status. Server reads are passive. Reuse bounded frontend polling initially,
invalidate after ingestion, and add subscriptions only when justified. Update
GraphQL, SDK, generated metadata, UI, and samples together as one breaking change.

## Execution boundary and installation experience

Only a project explicitly selected on disk through local `serve` or standalone
`collect run` supplies an executable collector plan. Both entry points validate
the same typed declarations and use the workspace lease/execution boundary.
Imported diagrams, cloud/database manifests, browser previews, and ordinary
GraphQL report mutations cannot register arbitrary commands. Remote configuration
does not become local execution authority.

Run commands as executable plus argv with explicit cwd, environment references,
and limits. Do not interpolate repository names, branch names, results, or browser
input into shell code. Inherited credentials are restricted to those needed by
the adapter; token values are never serialized into the SDK envelope or reports.
Adapters must also account for the tool's own config loading, hooks, network
access, and shell behavior. Path checks alone do not sandbox trusted project code.

A future local Run/Pause control accepts registered
`(manifest_id, section, collector_id)` identities only, bound to the active plan
and a process-scoped capability. Do not expose process execution
under today's shared writable local identity alone; check loopback binding,
Host/Origin, and a session capability. Hosted/non-loopback modes cannot activate
laptop collectors. Command-bearing browser saves require the same capability and
explicit local apply semantics. Preserve editor revision/conflict checks.

Proposed CLI experience, all powered by the same typed manifest declarations:

```sh
scryr collect doctor                 # explain missing/incompatible tools and auth
scryr collect list                   # show collectors, effective argv and schedules
scryr serve --watch                  # run the local UI/API and enabled schedules
scryr collect run --manifest services/api --section tests --collector unit
scryr collect status                # last attempts, freshness, next scheduled runs
```

Manual collection should contact the active local collector owner when present;
otherwise it takes the workspace lease and uses the same runner and persistence
path. Report-only input remains usable without launching a command.

No automatic installations on starting `serve`. `doctor` checks the active tool
environment and reports supported versions and installation guidance. Later,
`scryr collect install` can explicitly install declared, pinned tools through mise
using typed Forge declarations and generated private configuration. Never overwrite
an existing user mise file. Any required third-party configuration, such as
Nextest JUnit settings, must be generated from typed SDK settings when the user
wants a self-contained `index.scry`; do not quietly introduce another mandatory
hand-maintained config. Respect existing runner configuration when referenced.

## Implementation sequence and acceptance gates

1. **Replace SDK sections and freeze the new contract.** Define section-specific
   unions of concrete collectors; update `Manifest.__init__`, exports, model/schema
   generation, field refs, and Rust parsing. Rename `github` to `repository`, replace
   `cicd` with local `checks`, and route remote CI through Repository. Remove old
   section/source/provider classes and compatibility aliases. Acceptance: the new
   examples type-check; wrong-section collectors and invalid integration fields
   fail; Pyodide/native preview stays inert. Rewrite every maintained sample and
   authoring guide instead of preserving old snapshots.
2. **Replace observation storage and card reads.** Define the new observation,
   status, artifact, license, and projection models; introduce the database schema
   version/reset guidance, typed GraphQL operations, and six section view models.
   Remove obsolete report readers and Actions/provider special paths. Acceptance:
   recorded test/coverage/check results update the correct card without manifest
   re-upload, with correct organization/workspace/section/suite isolation.
3. **Local runtime and first vertical slice.** Add supervisor, scheduler, argv
   runner, workspace lease, per-run artifacts, local ingestion/spool, cancellation,
   diagnostics, and `collect` commands. Implement `GitStatusCollector`,
   `PytestCollector`, `JUnitReportCollector`, and one concrete local check collector.
   Acceptance: `serve --watch` shows repository context and local test/check results;
   missing tools and valid failed outcomes differ; server-only cannot execute;
   shutdown leaves no child processes.
4. **Dependencies with equal license/security support.** Add `SyftInventoryCollector`,
   `GrantLicenseCollector`, `GrypeScanCollector`, typed policy compilation, SBOM
   references, expression-aware license evidence, database freshness, and the three
   Dependencies panels. Acceptance: package/license/vulnerability fixtures render;
   unknown licenses remain visible; policy edits reevaluate cached inventory;
   advisory updates refresh security; incomplete scans never appear clean.
5. **Complete Repository, Metrics, Checks, and Performance.** Add the two GitHub
   collectors to Repository; complete concrete local check and JS/Rust test runners;
   implement `OpenMetricsCollector`, then optional `DockerStatsCollector` and
   `HyperfineBenchmarkCollector`. Delete the old PostHog/Prometheus server path,
   provider-query command, and browser metric hooks. Acceptance: one metrics endpoint
   populates Metrics without extra service configuration; rate/quantile results are
   correct or explicitly unavailable; Repository CI never appears in Checks; all
   six cards have useful waiting/error/stale states and accurate provenance.
6. **OSS onboarding and removal audit.** Ship typed examples, generated editor
   completion, doctor/install guidance, offline/resource behavior, retention,
   and standalone smoke coverage. Regenerate bundled SDK, metadata, GraphQL, and
   samples from the new sources. Remove unused files, exports, commands, generated
   old schema fields, docs, tests, and dependencies for replaced paths. Acceptance:
   a clean laptop with selected tools uses one authored `index.scry`; no old aliases,
   legacy renderers, or parallel provider systems remain. Consider explicit managed
   installation, remote result sync, alternative scanners, k6, Gitleaks, and act
   independently after this core is complete.

The integration owner owns `mise.toml`, lockfiles, shared contracts, and generated
artifacts. After the contract is concrete, SDK/runtime/UI work can use separate
writing worktrees and explicit file ownership. Existing provider branches are not
merged prerequisites; resolve any integration overlap in favor of this new design.

## Validation for implementation

- Python typing/Pydantic, Rust, and TypeScript fixtures for every section/collector
  pairing, wrong-section rejection, invalid options/policies/refs/schedules, stable
  identity under list reordering, and new-contract round trips. Old SDK APIs and
  serialized fields should fail rather than resolve through aliases.
- Deterministic scheduler/process tests with fake clocks and fixture executables:
  no overlap, debounce, missed ticks, bounded output, nonzero valid results,
  process-tree termination, changed-plan replacement, retries, crash replay, and
  stale artifacts. Unrelated source reloads must retain unchanged in-flight results
  without rerunning expensive collectors; changed inputs mark evidence outdated.
- Database/projection tests for idempotency, conflicting retries, out-of-order
  completion, section/workspace/tenant isolation, incompatible database detection,
  retention, and paginated package/license/vulnerability detail. Derived results
  must match inventory/policy revision. No database reset happens implicitly.
  Test queued/running/completed observations for one GitHub run alongside duplicate
  deliveries and delayed snapshots; idempotency must not suppress state changes.
- License fixtures for SPDX AND/OR/exception expressions, missing evidence,
  unknown expressions, policy changes, and versioned Grant output. Test that Syft
  alone still supplies useful license evidence and cannot imply a policy pass.
- Metric fixtures for supported exposition versions, content negotiation, labels,
  unit metadata, counter reset/first sample, histogram aggregation, missing buckets,
  stale scrapes, and cardinality/response limits. No Prometheus/PostHog server or
  account should be needed for the Metrics acceptance test.
- Execution-boundary tests reject missing/invalid capabilities, hostile Host/Origin,
  hosted/non-loopback execution, and registration through uploads or ordinary
  GraphQL writes. Only an authorized, successfully committed and published local
  editor revision activates a new plan. Test both serve and standalone manual
  execution, including section-qualified selectors. Passive reads never execute
  commands.
- Tool fixtures for supported versions, partial/unsupported scans, stale advisory
  data, absent credentials/tools, dirty worktrees, suites/coverage, and comparable
  benchmarks. Network tests are opt-in; core CI needs neither GitHub auth nor Docker.
- UI tests for all six example displays, changing/removing collectors, list order,
  lifecycle versus findings, outdated derived results, Repository CI versus local
  Checks, and draft/revision preservation. No static evidence fallback remains.
- Run `mise run verify:manifest`, `mise run verify:crystal`, and
  `mise run verify:map` during implementation; `mise run verify:editor` when editor
  integration changes. Generate GraphQL, manifest metadata, SDK bundles, and samples
  through existing mise tasks; review generated diffs, then run `mise run verify`
  including standalone release smoke checks. Use fake collectors in smoke tests.

The implemented SDK examples and tool setup guidance live in
`manifest/scryr/README.md` and `docs/src/content/docs/integrations.md`. The
`verify:collectors` smoke test exercises independent polling, manual collection,
and pause/resume against an isolated local database.
