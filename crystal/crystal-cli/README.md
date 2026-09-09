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

## Commands

```bash
scryr serve
scryr generate <target> --path index.scry
scryr auth <command>
```

Legacy positional generation still works for compatibility, but new docs and
scripts should use `--path`.

## `serve`

Runs the Scryr GraphQL server from the CLI.

```bash
scryr serve
scryr serve --sample mern --host 127.0.0.1 --port 8000
```

Server options:

```bash
--sample <name>       Explicit persisted artifact key to serve. No default sample.
--host <host>         Host interface to bind. Defaults to 127.0.0.1.
--port <port>         TCP port to bind. Defaults to 8000.
--auth-mode <mode>    Authentication mode: local or clerk.
```

`DATABASE_URL`, `SCRYR_SQLITE_PATH`, `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`,
`CLERK_SECRET_KEY`, and
`CORS_ALLOWED_ORIGINS` remain runtime environment settings for the embedded
server.

`scryr serve` defaults to local auth. Use `--auth-mode clerk` only for a hosted
Clerk-backed deployment.

## Shared Generate Options

These options are accepted by all `generate` targets:

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
the Rust CLI is compiled, `mise run sync:embedded-sdk` refreshes the crate-local
copy at `crystal/crystal-cli/static/scryr-sdk`, removing build and cache files.
`crystal/crystal-cli/build.rs` validates that copy and generates the embedded
asset list used by the binary. Changes to `manifest/scryr` therefore require
running the sync step before rebuilding the CLI.

Forge-backed targets also accept:

```bash
--forge <name-or-variable>
```

Use `--forge` when a manifest file defines multiple public `Forge` instances.

## `generate upload`

Executes the manifest, generates schema and diagram-scoped map artifacts, and
persists them through the configured GraphQL API.

```bash
scryr generate upload --path index.scry
```

When `SCRYR_GRAPHQL_URL` is unset, `generate upload` defaults to
`http://127.0.0.1:8000/graphql` and does not use a bearer token. Required for
hosted Clerk-backed uploads:

```bash
scryr auth login
```

Relevant environment variables:

```bash
SCRYR_GRAPHQL_URL
SCRYR_CLERK_ORG_ID
SCRYR_GIT_COMMIT_SHA
```

`generate upload` is the only generate target that writes to the Scryr backend.
The other targets print artifacts to stdout. Localhost, `127.0.0.1`, and `[::1]`
GraphQL endpoints use tokenless local auth; all other endpoints require a
cached Clerk OAuth token.

## `generate types`

Prints JSON metadata describing declared and runtime field types for public
top-level Scryr objects.

```bash
scryr generate types --path index.scry
```

This is useful for debugging manifests, validating `.scry` imports, and building
editor or inspection tooling.

## `generate schema`

Prints the serialized Pydantic JSON schema for Scryr's top-level models.

```bash
scryr generate schema --path index.scry
```

The output contains schemas for:

- `Manifest`
- `Forge`
- `Diagram`

## `generate mise`

Prints a selected `Forge` as `mise.toml`.

```bash
scryr generate mise --path index.scry --forge "MERN Forge"
```

It renders modeled Forge sections such as:

- `[tools]`
- `[env]`
- `[vars]`
- `[tasks]`
- `[settings]`
- `[plugins]`

## `generate compose`

Prints Docker Compose YAML for service tools declared in the selected `Forge`.

```bash
scryr generate compose --path index.scry --forge "MERN Forge"
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

## `generate devcontainer`

Prints a `devcontainer.json` for the selected `Forge`.

```bash
scryr generate devcontainer --path index.scry --forge "MERN Forge"
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

Use this before `generate upload`.

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
cargo run -p crystal-cli -- generate types --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest
cargo run -p crystal-cli -- generate mise --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest --forge "MERN Forge"
cargo run -p crystal-cli -- generate compose --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest --forge "MERN Forge"
cargo run -p crystal-cli -- generate devcontainer --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest --forge "MERN Forge"
```

Upload a sample to a local GraphQL server:

```bash
cargo run -p crystal-cli -- generate upload --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest
```

## Report GitHub Actions status

`scryr report-action-status --manifest-id services/api` reads a `workflow_run`
event from `GITHUB_EVENT_PATH` (or `--event-file`) and sends it to
`SCRYR_ENDPOINT` (or `--endpoint`) using `SCRYR_TOKEN` and
`SCRYR_CLERK_ORG_ID`. This command is implemented entirely in Rust.

Automatic branch selection uses `GITHUB_TOKEN` to check `main`, then `master`,
then falls back to the repository default branch. Override with `--branch`.
No workflow ID is required; use `--workflow-id` to restrict reporting.

## Operational reports

Use `scryr report tests`, `coverage`, `dependencies`, or `deployment` to publish
existing CI results without regenerating manifests. `scryr report actions` is
the grouped form of `report-action-status`. All reporters support `--dry-run`
and `--json`.

## Metrics on diagram load

Declare `Metrics(provider=PrometheusSource(...))` in `index.scry` to fetch Grafana
metrics when a diagram opens. Ordinary map polling does not fetch runtime metrics.
