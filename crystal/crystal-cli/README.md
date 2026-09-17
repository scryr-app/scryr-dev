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

When uploading with `scryr push` or `scryr serve`,
the diagram group is its parent folder relative to `--manifest-dir`. For example,
`--path services/catalog/index.scry` assigns the group `services/catalog`.
An entrypoint directly in the project root has no group; these diagrams appear
first in the map dropdown without a group heading.

```bash
scryr check
scryr format
scryr lint --fix
scryr push
scryr serve --watch
scryr collect list
scryr collect doctor
scryr collect run --manifest api --section tests
scryr collect status
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
scryr serve --no-collect
scryr serve --server-only --sample mern
```

By default, `serve` starts the embedded UI and GraphQL server, waits for readiness,
formats sources, runs all checks, uploads the validated artifact to that server,
and opens the UI. It starts the local collector owner for the typed declarations
in the validated manifest. Collector intervals and file watches run independently
of `--watch`, which reloads the manifest after source changes. Local loading always uses
the server's actual host and port, even when a cloud endpoint is configured.
Validation failures are printed in the terminal; the server stays available and
the previous valid diagram remains loaded. Without `--watch`, initial loading runs once.

Open the Python editor in the UI to load the selected diagram's actual `index.scry`.
**Run** executes the real Scryr SDK in an isolated Pyodide worker, then validates
the local source using the CLI's native checks. A successful run atomically
replaces the registered entrypoint and updates every diagram declared by it.
This works with plain `scryr serve`, with `--watch`, and on custom ports; no Vite
server or repository checkout is needed by an installed binary.

Syntax/validation errors leave the last saved file and diagram intact. External
edits are detected; a conflicting browser draft is retained and must be copied
or discarded with **Reload source** before saving. Supporting `.scry`/`.py` imports
are available to Python but are not editable through this single-file editor.
Local file editing requires a loopback bind address and the same-origin UI;
explicit loopback `CORS_ALLOWED_ORIGINS` also support Vite development.

In cloud/server-only mode, Run saves the organization's stored source snapshot
and all its diagram artifacts in one transaction; it does not write the server's
filesystem or commit to Git. Cloud write permissions are enforced by the server.
Pyodide and its pinned Python packages require access to the jsDelivr CDN on
first execution. A cancelled or timed-out Python run is not saved. Once saving
starts, wait for its result. If the local process is interrupted while committing,
restart `scryr serve` to regenerate artifacts from the authoritative disk source.

- `--server-only`: serve stored data without loading a local manifest or starting a collector owner.
- `--no-collect`: start the local UI and manifest loader with collection paused; use `scryr collect resume` to enable it.
- `--no-format`: verify formatting without changing source files.
- `--no-open`: do not launch a browser.
- `--host`: bind interface (default `127.0.0.1`).
- `--port`: TCP port (default `8000`).
- `--sample`: persisted artifact selection; no default sample.
- `--auth-mode`: `local` (default) or `clerk`.

`DATABASE_URL`, `SCRYR_SQLITE_PATH`, `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`,
`CLERK_SECRET_KEY`, and `CORS_ALLOWED_ORIGINS` remain server environment settings.

## Shared manifest options

These options are accepted by check, format, lint, push, serve, export, inspect,
and collect subcommands:

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
print artifacts to stdout; progress messages go to stderr. Localhost, `127.0.0.1`, and `[::1]`
GraphQL endpoints use tokenless local auth; all other endpoints require `SCRYR_TOKEN` or a cached Clerk OAuth token.

Configure the upload endpoint with `--endpoint` or `SCRYR_ENDPOINT`; otherwise
`push` uses loopback and `PORT` or port 8000. Local collection writes to the
local SQLite database used by `serve`; an upload endpoint does not authorize
remote command execution.

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

## Declare laptop evidence in `index.scry`

Use concrete SDK collectors directly in each section. There is no separate
collector config file or global collector registry:

```python
from datetime import timedelta
from scryr import (
    Diagram, GitHubActionsCollector, GitStatusCollector, GrantLicenseCollector,
    GrypeScanCollector, JUnitReportCollector, LicensePolicy, Manifest,
    OpenMetricsCollector, PytestCollector, RuffCheckCollector, SbomRef,
    Schedule, SyftInventoryCollector,
)

api = Manifest(
    manifest_id="api",
    name="API",
    repository=[
        GitStatusCollector(),
        GitHubActionsCollector(repository="your-org/your-repo"),
    ],
    checks=[RuffCheckCollector(paths=["src"])],
    tests=[
        PytestCollector(id="unit", paths=["tests"]),
        JUnitReportCollector(id="integration", files=["results/integration.xml"]),
    ],
    metrics=[OpenMetricsCollector(
        endpoint="http://127.0.0.1:8080/metrics",
        schedule=Schedule(startup=True, every=timedelta(seconds=30)),
    )],
    dependencies=[
        SyftInventoryCollector(id="packages"),
        GrantLicenseCollector(
            sbom=SbomRef(collector_id="packages"),
            policy=LicensePolicy(allow=["MIT", "Apache-2.0"]),
        ),
        GrypeScanCollector(sbom=SbomRef(collector_id="packages")),
    ],
)
system = Diagram(name="Local development", manifests=[api])
```

Constructing these objects only validates and serializes settings. `manifest_id`
and collector `id` identify the source across runs; set different IDs for multiple
instances of the same integration. Relative collector paths are contained within
`--manifest-dir`. `SbomRef` binds both license and vulnerability evidence to the
exact Syft inventory; use its optional `manifest_id` for another declared manifest.

| Section / card | Concrete integrations | Installed tools |
| --- | --- | --- |
| `repository` / Repository | `GitStatusCollector`, `GitHubPullRequestsCollector`, `GitHubActionsCollector` | `git`, `gh` |
| `checks` / Checks | `RuffCheckCollector`, `BiomeCheckCollector`, `CargoClippyCollector`, `MiseTaskCollector` | `ruff`, `biome`, `cargo`, `mise` |
| `metrics` / Metrics | `OpenMetricsCollector`, `DockerStatsCollector` | HTTP endpoint, `docker` |
| `tests` / Tests | `PytestCollector`, `VitestCollector`, `NextestCollector`, `JUnitReportCollector`, `LcovCoverageCollector`, `CoberturaCoverageCollector` | `pytest`, `vitest`, `cargo-nextest`, or existing files |
| `dependencies` / Dependencies | `SyftInventoryCollector`, `GrantLicenseCollector`, `GrypeScanCollector` | `syft`, `grant`, `grype` |
| `performance` / Performance | `HyperfineBenchmarkCollector` | `hyperfine` and the declared executable |

Repository includes remote CI workflow runs. Checks records local commands.
Metrics scrapes the endpoint directly; `CounterRate`, `GaugeSeries`, and
`HistogramPercentile` provide typed selections without a provider service or
query language. Dependencies presents inventory, licenses, and vulnerabilities
with equal prominence. Grant enumerates the shared inventory; Scryr applies the
typed policy to the original SPDX `AND`, `OR`, and `WITH` expressions. Unknown
license evidence requires review unless the explicit policy denies it.

A benchmark is an explicit argv declaration, for example:

```python
from scryr import Command, HyperfineBenchmarkCollector

benchmark = HyperfineBenchmarkCollector(
    command=Command(executable="./target/release/api", args=["--self-test"]),
    warmup=2,
    runs=10,
)
# Put benchmark in a Manifest's performance=[benchmark] list.
```

Install collector tools yourself in the project environment or `PATH`.
Scryr checks `.venv/bin`, `node_modules/.bin`, and `PATH` and reports missing or
incompatible tools. It does not auto-install collector CLIs or log into GitHub.
Run `gh auth login` yourself when using GitHub collectors. `ToolRequirement`
constrains the integration's version. `EnvRef` supplies explicitly named machine
environment variables without putting secret values in serialized declarations.
The managed Python runtime described above remains part of manifest execution.

## Control local collection

```bash
scryr collect list
scryr collect doctor
scryr collect run --manifest api --section tests --collector unit
scryr collect status
scryr collect pause
scryr collect resume
```

- `list` validates the manifest and prints declared collectors and settings.
- `doctor` checks executable availability, optional version requirements, and
  applicable authentication. It does not execute tests, scans, or benchmarks.
- `run` requests a manual run from the active `serve` owner. Without an active
  owner, it validates the manifest and performs a one-shot collection locally.
  The selected schedules must enable `manual`.
- `status` shows current collector lifecycle and freshness settings, or the last
  saved status when the owner is stopped.
- `pause` cancels active child processes and pauses collection for the workspace.
- `resume` re-enables the active owner's declared schedules.

Use `--manifest` with a stable `manifest_id`, `--section` with one of the six
section names, and `--collector` with a collector ID to narrow `list`, `doctor`,
and `run`. Pause and resume require an active owner and apply to the whole
workspace. Use the same project root and database configuration as `serve`.

Git and lightweight metrics poll by default. GitHub collectors use slower
intervals. Syft observes startup and lockfile changes; Grant and Grype follow the
shared inventory, with periodic Grype rescanning. Tests, checks, and benchmarks
default to manual execution. Existing JUnit/LCOV/Cobertura collectors read and
watch files without running their producers. Override these behaviors using a
typed `Schedule` in `index.scry`.

Starting `scryr serve --no-collect` pauses this execution owner until an explicit
`collect resume`. Browser previews, manifest uploads, and ordinary GraphQL
writes cannot register or run laptop commands. `serve --server-only` does not
start a collection owner.

## Local storage, provenance, and freshness

Collection requires a local file-backed SQLite database. Its default path is
`.scryr/scryr.db`; use `SCRYR_SQLITE_PATH` or a SQLite `DATABASE_URL` to select
another file. Relative database paths resolve from the command's working
directory. Use an absolute path when running commands from different directories.
Remote Turso/PostgreSQL and in-memory databases are not collection targets.

The workspace identity combines the canonical project root and database path.
A local lock and private control socket ensure one execution owner for that
workspace. Temporary run artifacts, pending observations, and cached status live
under `<manifest-dir>/.scryr/collection/<workspace>`, or under the corresponding
`--scryr-dir` location. `--scryr-dir` does not change the database path.

Normalized observations are persisted through the shared core and available to
the map through GraphQL. They retain collector identity, configuration/input
revision, applicable Git context, tool version, and collection time. Imported
JUnit and coverage artifacts retain their source modification time separately;
for multiple files the oldest source time determines age. A successful import
of an old file therefore remains old evidence. Scanner results retain inventory
identity and database provenance; missing or partial scans cannot imply a clean
Dependencies result.

The UI distinguishes current, stale, outdated, partial, and missing evidence
from execution states such as running or missing-tool. A failed or cancelled
attempt leaves the last successful result and its age visible. Editing a
collector invalidates incompatible observations, and benchmark comparisons
require a compatible baseline. Repository results, local tests, dependency
scans, and measurements keep their own source scope.

See [the SDK integration guide](../../docs/src/content/docs/integrations.md)
for examples of all six cards and their display states.

## Export and inspect

`export json` prints checked diagram JSON. `export mise`, `export compose`, and
`export devcontainer` operate on Forges and do not require a Diagram.
`inspect types` prints runtime metadata; use `check` for static type checking.
`inspect schema` prints the SDK's JSON schema.
