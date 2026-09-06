# Contributing

Scryr is a monorepo with three projects:

- `manifest`: Python SDK, sample `.scry` manifests, and Python tests.
- `crystal`: Rust crates for the GraphQL server, persistence layer, and CLI.
- `map`: React/Three.js frontend.

Use the root [mise.toml](mise.toml) for all development tasks. 
## First-Time Setup

Install toolchains and dependencies from the repository root:

```bash
mise install # installs system level dependencies like python and node
mise run install-dependencies # installs node, rust, and python coding packages
```

All of the contributing tasks are coordinated with mise tasks.  Run `mise tasks ls` to see the main task groups.


Personal settings and secrets belong in untracked `mise.local.toml`. Do not
commit credentials, generated secrets, or personal deployment config.

## Static Analysis

Static analysis is split into formatting, linting, and type checks:

```bash
mise run format:check
mise run lint
mise run type-check
```

For the whole static-analysis pass:

```bash
mise run static
```

Project-specific checks are available when you want a narrower loop:

```bash
mise run format:check:rust
mise run lint:python
mise run type-check:map
```

Auto-fix what the tools can safely fix:

```bash
mise run fix
```

## Tests

Tests run in two phases: unit tests first, then integration tests.

```bash
mise run test:unit
mise run test:integration
```

The full test command runs both phases in order:

```bash
mise run test
```

Narrow test commands:

```bash
mise run test:unit:rust
mise run test:integration:rust
mise run test:unit:python
mise run test:integration:python
mise run test:unit:map
```

## Development And Deployment Modes

Use the local no-cloud mode for normal contribution work. The cloud-backed and
deployment modes are only needed when you are testing Clerk, hosted database
behavior, embedded release assets, or Fly deployment packaging.

| Mode                           | Purpose                                                                                        | Services                                                                                      | Primary Commands                                                                          |
| ------------------------------ | ---------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| Local no-cloud development     | Default contributor workflow for backend, frontend, SDK, and sample work.                      | Local Rust server, SQLite under `.cache/scryr`, local token auth, Vite dev server.            | `mise run setup:dev:local`, then `mise run run:server:local` and `mise run run:ui:local` |
| Local cloud-backed development | Test Clerk auth and Turso from a local checkout.                                               | Local Rust server, configured Turso, Clerk auth, Vite dev server with Clerk enabled.          | `mise run dev:cloud`, then `mise run run:server:cloud` and `mise run run:ui:cloud`       |
| Self-contained OSS CLI         | Build or install the release-style CLI with embedded map UI, sample artifacts, and Python SDK. | Single local `scryr` binary; runtime state is provisioned under Scryr-managed local state.    | `mise run build:oss`, `mise run install:oss`                                              |
| Hosted cloud deployment        | Build Clerk-enabled embedded assets and deploy the server app.                                 | Fly app from `crystal/fly.toml`, Clerk auth, Turso storage.                                  | `mise run deploy:secrets`, `mise run deploy:cloud`                                        |
| Docker image                   | Build the server image locally for packaging checks.                                           | Local Docker image from the Crystal context.                                                  | `mise run build:docker`                                                                   |

## Local Development Without Cloud Services

This is the default contributor workflow. It uses local SQLite, local auth, and
no Clerk or cloud database credentials.

Prepare the stack:

```bash
mise run setup:dev:local
```

Run the server and frontend in separate terminals:

```bash
mise run run:server:local
mise run run:ui:local
```

The server is available at `http://127.0.0.1:8000`, and the frontend is
available at `http://localhost:3000`.

Regenerate frontend GraphQL types:

```bash
mise run codegen-graphql-requests
```

Populate the local SQLite database with the bundled sample manifests:

```bash
mise run run:cli:samples
```

That task reuses a running local server, or starts one temporarily if nothing is
listening on `127.0.0.1:$PORT`.

## Local Development With Cloud Services

Use this path only when you need to test Clerk auth or a cloud database from a
local checkout.

Set these values in your shell or in untracked `mise.local.toml`:

```toml
[env]
TURSO_DATABASE_URL = "libsql://your-database.turso.io"
TURSO_AUTH_TOKEN = "..."
CLERK_SECRET_KEY = "sk_..."
VITE_CLERK_PUBLISHABLE_KEY = "pk_..."
VITE_GRAPHQL_ENDPOINT = "http://localhost:8000/graphql"
```

Then prepare and run the cloud-backed local stack:

```bash
mise run dev:cloud
mise run run:server:cloud
mise run run:ui:cloud
```

Cloud mode sets `AUTH_MODE=clerk` for the server and
`VITE_SCRYR_AUTH_MODE=clerk` for the frontend.

## Build And Install The Open-Source CLI

The open-source CLI build is self-contained from a user's perspective: the Rust
binary embeds the map UI, bundled sample artifacts, and the Python SDK copy that
Scryr provisions through its own `.scryr` state on first manifest execution.

Build it:

```bash
mise run build:oss
```

Install it on your PATH:

```bash
mise run install:oss
```

`install:oss` builds the self-contained release binary and copies it to
`${SCRYR_CLI_INSTALL_DIR:-$HOME/.local/bin}/scryr`.  Make sure `~/.local/bin` is on your `PATH`, or start a fresh
shell after `install:oss` updates your shell profile.

Smoke test the installed binary:

```bash
command -v scryr
scryr --version
scryr serve
```

## Cloud Deployment

The cloud deployment uses the existing [crystal/fly.toml](crystal/fly.toml)
configuration and Clerk auth.

First-time Fly setup:

```bash
flyctl auth login
flyctl apps list
```

Set deployment secrets from your local environment:

```bash
export CLERK_SECRET_KEY="sk_..."
export TURSO_DATABASE_URL="libsql://your-database.turso.io"
export TURSO_AUTH_TOKEN="..."
mise run deploy:secrets
```

Build Clerk-enabled embedded assets and deploy:

```bash
export VITE_CLERK_PUBLISHABLE_KEY="pk_..."
export VITE_GRAPHQL_ENDPOINT="/graphql"
mise run deploy:cloud
```

`deploy:cloud` builds cloud-mode embedded frontend assets, refreshes bundled
sample artifacts, optionally updates Fly secrets from `CLERK_SECRET_KEY` and
Turso variables when present, and runs `flyctl deploy --config crystal/fly.toml`.

## Pull Requests

Before opening a pull request:

- Keep changes focused on one behavior or documentation area.
- Add or update tests for user-visible behavior changes.
- Run `mise run static` and the relevant `mise run test:*` commands.
- Run `mise run build:oss` when touching release packaging, embedded assets, or
  CLI install behavior.

## Reporting Issues

When filing a bug, include:

- Scryr version or commit SHA.
- Operating system and tool versions when relevant.
- Steps to reproduce.
- Expected behavior and actual behavior.
- Logs or screenshots if they clarify the issue.

## Generated Files

Generated GraphQL TypeScript output lives in `map/src/graphql/generated.ts`.
Regenerate it with:

```bash
mise run codegen-graphql-requests
```
