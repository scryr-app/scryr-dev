# Scryr CLI

`scryr` executes `.scry` or Python manifest files and turns their public
`Manifest`, `Diagram`, and `Forge` objects into artifacts for the Scryr map and
developer tooling.

From the workspace, run it with Cargo:

```bash
cargo run -p crystal-cli -- <command>
```

After installation, use the binary directly:

```bash
scryr <command>
```

## Primary workflows

Run these commands from your own repository. The default entrypoint is `index.scry`.
Use `--path` for a different file and `--manifest-dir` for a different project root.
Paths passed to `--path` are relative to `--manifest-dir` (or absolute).

```bash
scryr check
scryr format
scryr lint --fix
scryr push
scryr serve --watch
scryr report tests --run-id "$GITHUB_RUN_ID" --observed-at "$RESULTS_COMPLETED_AT"
scryr report actions
scryr query --list
scryr query request_latency --json
```

## Check, format, and lint

`scryr check` checks formatting, lint, and Python types, then executes the manifest
once, serializes JSON, and validates Scryr diagram rules. It requires at least one
public `Diagram`, unique diagram identifiers, and valid references to public
`Manifest` objects. Failures identify the stage and return a nonzero exit status.
Check does not rewrite source files or contact Scryr/provider APIs. Manifest
execution itself runs user Python code and may have whatever effects that code defines.

`scryr format` rewrites the entrypoint and reachable local `.scry`/`.py` imports.
`scryr format --check` reports formatting differences without writing.
`scryr lint` reports lint errors; `--fix` applies only safe fixes. Formatting and
linting do not execute user code or format unrelated Python files. Existing Ruff
configuration is honored, and `.scry` files are treated as Python. Unused-import
warnings are suppressed because public imports can export Scryr objects. Hyphenated Scryr
declarations retain their original names. Type checking uses temporary Python
mirrors to resolve `.scry` imports and maps diagnostics back to original paths.

Ruff 0.16.6 and ty 0.0.78 are pinned in the managed runtime, matching this repository's
lockfile. Tool provisioning can download dependencies on first use.

## Serve locally

```bash
scryr serve
scryr serve --watch --port 9000
scryr serve --no-format --no-open
scryr serve --server-only --sample mern
```

By default, `serve` starts the embedded UI and GraphQL server, waits for readiness,
formats sources, runs all checks, uploads the validated artifact to that server,
and opens the UI. `--watch` repeats on source changes. Local loading always uses
the server's actual host and port, even when a cloud endpoint is configured.
Validation failures are printed in the terminal; the server stays available and
the previous valid diagram remains loaded. Without `--watch`, initial loading runs once.

- `--server-only`: do not read, format, execute, or upload a local manifest. Use in deployments.
- `--no-format`: verify formatting without changing source files.
- `--no-open`: do not launch a browser.
- `--host`: bind interface (default `127.0.0.1`).
- `--port`: TCP port (default `8000`).
- `--sample`: persisted artifact selection; no default sample.
- `--auth-mode`: `local` (default) or `clerk`.

`DATABASE_URL`, `SCRYR_SQLITE_PATH`, `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`,
`CLERK_SECRET_KEY`, and `CORS_ALLOWED_ORIGINS` remain server environment settings.

## Shared manifest options

These options are accepted by check, format, lint, push, serve, export, inspect, and query:

```bash
--path <file>          Manifest file to execute. Defaults to index.scry.
--manifest-dir <dir>   Manifest project root. Defaults to the current directory.
--scryr-dir <dir>      Scryr-managed local state directory.
```

For local execution, Scryr manages the Python runtime itself. On first use it
installs Scryr's pinned `uv` from Astral's official standalone installer into
`<scryr-dir>/bin`, installs uv-managed Python into `<scryr-dir>/python`, creates
per-project venvs under `<scryr-dir>/python-envs`, and keeps the package cache
under Scryr-managed state. It does not use or modify a system `uv` or system
Python installation.

Local generation uses the embedded Python SDK by default, even when the manifest
directory contains an unrelated `pyproject.toml`. Add a `scryr.toml` file directly
inside `--manifest-dir` to explicitly use that directory's Python project instead.
An empty `scryr.toml` is sufficient; its contents are not currently parsed.

Project mode still requires `pyproject.toml` with the Python `scryr` dependency
and an up-to-date `uv.lock`, because uv uses those files for
`uv sync --no-dev --locked`. `scryr.toml` selects this mode; it does not replace
uv's dependency metadata. Without it, Scryr runs its embedded adapter using
`uv run --no-project` and a dedicated script environment.

### Embedded Python SDK

The standalone OSS CLI packages the Python SDK from `manifest/scryr`. Before
the Rust CLI is compiled, `mise run internal:embed:manifest` refreshes the crate-local
copy at `crystal/crystal-cli/static/scryr-sdk`, removing build and cache files.
`crystal/crystal-cli/build.rs` validates that copy and generates the embedded
asset list used by the binary. Changes to `manifest/scryr` therefore require
running the sync step before rebuilding the CLI.

Forge exports also accept:

```bash
--forge <name-or-variable>
```

Use `--forge` when a manifest file defines multiple public `Forge` instances.

## `push`

Executes the manifest, generates schema and diagram-scoped map artifacts, and
persists them through the configured GraphQL API.

```bash
scryr push --path index.scry
```

When no endpoint is configured, `push` defaults to
`http://127.0.0.1:8000/graphql` and uses tokenless local authentication unless `SCRYR_TOKEN` is explicitly set. Required for
hosted Clerk-backed uploads:

```bash
scryr auth login
```

Relevant environment variables:

```bash
SCRYR_ENDPOINT
SCRYR_CLERK_ORG_ID
SCRYR_GIT_COMMIT_SHA
```

`push` validates before uploading and reuses the generated JSON without executing
the manifest again. It prints the deployment UI URL. Export and inspect commands
print artifacts to stdout; progress and deprecation messages go to stderr. Localhost, `127.0.0.1`, and `[::1]`
GraphQL endpoints use tokenless local auth; all other endpoints require `SCRYR_TOKEN` or a cached Clerk OAuth token.

Endpoint precedence is `--endpoint` (alias `--graphql-url`), `SCRYR_ENDPOINT`,
legacy `SCRYR_GRAPHQL_URL`, then loopback using `PORT` or port 8000. This applies
to push, report, and query.

## `inspect types`

Prints JSON metadata describing declared and runtime field types for public
top-level Scryr objects.

```bash
scryr inspect types --path index.scry
```

This is useful for debugging manifests, validating `.scry` imports, and building
editor or inspection tooling.

## `inspect schema`

Prints the serialized Pydantic JSON schema for Scryr's top-level models.

```bash
scryr inspect schema --path index.scry
```

The output contains schemas for:

- `Manifest`
- `Forge`
- `Diagram`

## `export mise`

Prints a selected `Forge` as `mise.toml`.

```bash
scryr export mise --path index.scry --forge "MERN Forge"
```

It renders modeled Forge sections such as:

- `[tools]`
- `[env]`
- `[vars]`
- `[tasks]`
- `[settings]`
- `[plugins]`

## `export compose`

Prints Docker Compose YAML for service tools declared in the selected `Forge`.

```bash
scryr export compose --path index.scry --forge "MERN Forge"
```

Currently supported service tools:

- `postgres` / `postgresql`
- `mongodb` / `mongo`
- `redis`
- `mysql`
- `mariadb`

Tool versions become Docker image tags. For example:

```python
Forge(
    name="MERN Forge",
    tools={
        "mongodb": "8",
        "redis": "7",
    },
)
```

Generates services using `mongo:8` and `redis:7`.

## `export devcontainer`

Prints a `devcontainer.json` for the selected `Forge`.

```bash
scryr export devcontainer --path index.scry --forge "MERN Forge"
```

The generated devcontainer:

- uses `mcr.microsoft.com/devcontainers/base:ubuntu`
- installs mise in `postCreateCommand`
- runs `mise install`
- runs `mise run install` when the Forge defines an `install` task
- copies primitive Forge env values into `containerEnv`
- adds VS Code extensions for known tools such as Node, Python, Rust, and Terraform

## `auth login`

Starts the Clerk OAuth login flow and stores local CLI tokens for hosted
uploads. Local no-cloud uploads do not require this command.

```bash
scryr auth login
```

Use this before `push`.

Optional environment overrides:

```bash
SCRYR_CLERK_PUBLISHABLE_KEY
SCRYR_CLERK_OAUTH_CLIENT_ID
SCRYR_CLERK_OAUTH_SCOPES
```

## `auth whoami`

Prints the currently authenticated Clerk subject.

```bash
scryr auth whoami
```

If the stored access token is stale and a refresh token is available, the CLI
attempts to refresh it.

## `auth logout`

Deletes local Scryr CLI OAuth state.

```bash
scryr auth logout
```

## Examples

Run against the MERN sample:

```bash
cargo run -p crystal-cli -- inspect types --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest
cargo run -p crystal-cli -- export mise --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest --forge "MERN Forge"
cargo run -p crystal-cli -- export compose --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest --forge "MERN Forge"
cargo run -p crystal-cli -- export devcontainer --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest --forge "MERN Forge"
```

Upload a sample to a local GraphQL server:

```bash
cargo run -p crystal-cli -- push --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest
```

## Report existing results

Declare a stable `manifest_id` and optional typed input settings:

```python
from scryr import ActionsReportSource, CICD, Diagram, Manifest, TestReportSource, Tests

api = Manifest(
    manifest_id="services/api",
    name="API",
    tests=Tests(source=TestReportSource(files=["results/junit.xml"], suite="unit")),
    cicd=CICD(source=ActionsReportSource(workflow_id=42, branch="main", jobs_file="results/jobs.json")),
)
diagram = Diagram(name="System", manifests=[api])
```

```bash
scryr report tests --manifest api --run-id "$GITHUB_RUN_ID" --observed-at "$RESULTS_COMPLETED_AT"
scryr report actions --manifest api --event-file workflow-run.json
```

Artifact paths in declarations are relative to the entrypoint. Explicit flags
such as `--file`, `--format`, `--suite`, `--workflow-id`, `--branch`, and `--jobs-file`
override declared defaults. Multiple manifests require `--manifest` (public variable,
name, or stable ID). Reporting executes the manifest to resolve its declarations;
it does not run tests, regenerate diagrams, or automatically push diagrams.

Explicit `--manifest-id` without `--path` or `--manifest` retains the existing
Python-free reporting mode. Test reporting still requires a run ID and a stable
source completion time (`--observed-at`); `GITHUB_RUN_ID` supplies the former in CI.

Actions reporting reads `workflow_run` events from `GITHUB_EVENT_PATH` or
`--event-file`. Automatic branch selection uses `GITHUB_TOKEN` to check main,
master, then the repository default; `--branch` or the declaration avoids that lookup.
The optional jobs file is a complete GitHub jobs API response with `total_count`
and `jobs`; combine all pages first. Every job must belong to the exact workflow
run and attempt. Jobs enrich existing run history, and identical retries are idempotent.

The existing `report coverage`, `report dependencies`, and `report deployment`
commands remain supported. All reporters support `--dry-run` and `--json`.

## Query declared providers

```bash
scryr query --list
scryr query request_latency --manifest api
scryr query page_views --json --endpoint https://your-scryr.example/graphql
```

Query names come from `metrics.provider.queries` and `analytics.queries` in the
local manifest. Listing executes the manifest but makes no provider request.
Duplicate query names require a manifest selector; use `--provider prometheus` or
`--provider posthog` when both providers on one manifest use the same query name. Queries execute through a
running Scryr server and its organization-scoped `SCRYR_METRICS_CONNECTIONS_FILE`
connections; they do not publish the local manifest. Provider failures return a
nonzero exit status. JSON output includes timestamps, status, values, and units.
Grafana support uses Prometheus-compatible data sources; PostHog uses declared
HogQL aggregates. Credentials remain in server connections, outside artifacts.

## Export and inspect

`export json` prints checked diagram JSON. `export mise`, `export compose`, and
`export devcontainer` operate on Forges and do not require a Diagram.
`inspect types` prints runtime metadata; use `check` for static type checking.
`inspect schema` prints the SDK's JSON schema.

## Migration

| Deprecated command | Canonical command |
| --- | --- |
| `generate upload` | `push` |
| `generate types` | `inspect types` |
| `generate schema` | `inspect schema` |
| `generate mise` | `export mise` |
| `generate compose` | `export compose` |
| `generate devcontainer` | `export devcontainer` |
| `report-action-status` | `report actions` |

Compatibility commands emit a deprecation warning on stderr. The hidden
`generate artifact-json` release interface retains its existing output behavior;
new users should use `export json`. Legacy uploads now undergo full validation,
including the required Diagram rule. Existing persisted artifacts remain readable.
Deployments that previously used `serve` must add `--server-only` to retain
server-only behavior. Docker and repository server scripts have been updated.
