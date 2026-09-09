# Operational reports through the Scryr CLI

`report` updates durable observations for a stable `manifest_id`, independently
of `.scry` generation. Supported commands are `actions`, `tests`, `coverage`,
`dependencies`, and `deployment`. `report-action-status` remains compatible.

## Shared behavior

New reporters require `--manifest-id`, `--run-id`, and `--observed-at` (RFC3339).
Run ID and attempt default from `GITHUB_RUN_ID` and `GITHUB_RUN_ATTEMPT` in CI.
Commit and branch metadata default from `GITHUB_SHA` and `GITHUB_REF_NAME`.
Use the original completion/snapshot time for retries, not the retry time.

`--endpoint` or `SCRYR_ENDPOINT` selects Crystal; the default is
`http://127.0.0.1:8000/graphql`. Local endpoints need no token. Hosted endpoints
use `SCRYR_TOKEN` or the existing `scryr auth login` session. Set
`SCRYR_CLERK_ORG_ID` for organization selection. GitHub collection uses a separate
`GITHUB_TOKEN`. Credential-bearing requests do not follow redirects.

`--dry-run` validates and prints the normalized JSON without contacting Crystal.
Dependabot dry runs still fetch GitHub when no `--file` is supplied; Actions dry
runs still discover the branch unless `--branch` is supplied. `--json` produces
machine-readable upload receipts (`recorded: true/false`). Nonzero exit means
reporting failed, not that the reported test suite or deployment failed.

Identical payload retries are deduplicated atomically. New observations append
history. Current values are ordered by source timestamp, attempt, and a stable
fingerprint tie-breaker; delivery order does not control current status.
Observations are organization-scoped and survive manifest regeneration/restarts.
`reportHistory(manifestId:, limit:, offset:)` pages through the history (limit
1–100). `ScryrClient.record_report()` and `.report_history()` expose the same API.

## Tests

```bash
scryr report tests --manifest-id northwind-commerce/api \
  --file test-results/integration-healthy.xml --format junit \
  --suite integration-healthy --run-id local-1 \
  --observed-at 2026-09-09T04:00:00Z
```

JUnit XML supports nested suites and counts testcase elements once. Errors and
skips remain distinct from failures. Duration is cumulative testcase time, not
wall-clock duration. Summary-only reports claiming nonzero tests are rejected.
Multiple `--file` arguments combine nonoverlapping files; duplicate testcase
identities are rejected. Report matrix shards with `--shard`, and keep unrelated
suites under different `--suite` values. Each gets its own map card. Cross-suite
and cross-shard totals are intentionally not inferred because scopes can overlap
or represent different commits. Publish a merged complete suite when a single
overall total is required. No flakiness is inferred from one result.

Publish main-branch results from a main-branch job, or include the branch in
`--suite` when reporting multiple branches. Branch metadata alone is not a
filter for tests/coverage. The Actions reporter retains its branch filtering.

## Coverage

```bash
scryr report coverage --manifest-id northwind-commerce/api \
  --file coverage.xml --format cobertura --suite unit \
  --run-id local-1 --observed-at 2026-09-09T04:00:00Z
```

LCOV uses `--format lcov`. Both formats merge source line identities across input
files and store covered/total counts. Zero executable lines means unavailable
coverage. Trends compare the prior run in the same scope, branch, and format
only when the executable-line total matches. Per-file details are used during
import but are not stored as source code.

## Dependabot

```bash
scryr report dependencies --manifest-id northwind-commerce/api \
  --repository scryr-app/ex-northwind-commerce --manifest-path api/uv.lock \
  --run-id security-snapshot-1 --observed-at 2026-09-09T04:00:00Z
```

Use a GitHub App token or PAT with **Dependabot alerts: read** repository
permission. Do not assume the workflow's default token has this access.
Collection fetches every page and reports nothing if a page fails. For offline
replay, pass `--file dependabot-alerts.json`: a complete GitHub alert array,
including open, fixed, and dismissed states. An empty array is an explicit
successful zero-alert snapshot, not a substitute for a collection failure.

The repository and exact dependency manifest path define the scope. Use the path
GitHub returns, which may differ from the project's preferred lockfile path.
Alerts outside that path are excluded. The map displays open counts by severity;
`vulnerableDeps` counts distinct ecosystem/package/path combinations. Inventory,
outdated counts, update lag, and license compliance are left unknown unless
supplied by another source. Historical snapshots retain resolved alerts.

GitHub API reference: https://docs.github.com/en/rest/dependabot/alerts

## Deployments

```bash
scryr report deployment --manifest-id northwind-commerce/api \
  --environment staging --status success --version abc123 \
  --run-id deploy-1 --observed-at 2026-09-09T04:00:00Z \
  --report-url https://github.com/example/repo/actions/runs/123
```

Accepted states: `pending`, `in_progress`, `success`, `failure`, `inactive`.
Each environment keeps its own current observation. Staging and prod/production
also update the existing deployment summary fields; they do not change build
status. Use one canonical name for production. Report cards show provenance,
source time, and optional links. Reports older than seven days are labeled with
their age; there is no implicit service-specific freshness SLA.

## Reproduce the Northwind local verification

From the `scryr-dev` root, with the sibling `ex-northwind-commerce` checkout and
its API development environment installed:

```bash
cargo build --manifest-path crystal/Cargo.toml -p crystal-cli
mkdir -p .scryr/local-reports
SCRYR_SQLITE_PATH="$PWD/.scryr/local-reports/scryr.db" \
  ./crystal/target/debug/scryr serve --auth-mode local --port 8001
```

In another terminal:

```bash
../ex-northwind-commerce/api/.venv/bin/pytest \
  -c ../ex-northwind-commerce/api/pyproject.toml \
  ../ex-northwind-commerce/api/tests -p no:cacheprovider \
  --junitxml=.scryr/local-reports/northwind-unit.xml \
  --cov=northwind_api --cov-report=xml:.scryr/local-reports/northwind-coverage.xml
python3 docs/examples/northwind-reporting-demo.py
```

The demo uploads Northwind, reports actual API tests and coverage, and submits
**synthetic** Dependabot and staging deployment fixtures. It checks duplicate
delivery, delayed delivery, and regeneration, then writes
`.scryr/local-reports/verified-summary.json`. It expects the current Northwind
suite's 16 passing tests. Open http://127.0.0.1:8001 and select **Northwind Commerce
Diagram**. Build refreshed embedded frontend assets with `mise run build:oss`
when rebuilding from source for UI changes.

## Follow-on adapters

SBOM inventory, SARIF security findings, benchmarks, and periodic runtime
summaries remain subsequent adapters from the plan. This release establishes
the four delivery steps: common reporting, tests/coverage, Dependabot, and
deployments. It does not run a monitoring agent or scan repositories itself.
