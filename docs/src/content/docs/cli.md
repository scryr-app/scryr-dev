---
title: CLI reference
description: Check, format, serve, push, sync, inspect, export, report, and query Scryr manifests.
---

Run commands from the repository containing `index.scry`.

## Primary workflows

```sh
scryr check
scryr format
scryr lint --fix
scryr push
scryr serve --watch
scryr sync github
scryr report tests --run-id "$GITHUB_RUN_ID" --observed-at "$RESULTS_COMPLETED_AT"
scryr report actions
scryr query --list
scryr query request_latency --json
```

## Check, format, and lint

### `scryr check`

Checks formatting, lint, Python types, manifest execution, serialization, and Scryr diagram rules. It requires at least one public `Diagram`, unique diagram identifiers, and valid references to public `Manifest` objects.

`check` does not rewrite source or contact Scryr/provider APIs. Manifest execution itself runs user Python code and can have whatever side effects that code defines.

### `scryr format`

Rewrites the entrypoint and reachable local `.scry`/`.py` imports. Use `scryr format --check` for a non-writing check.

### `scryr lint`

Reports lint errors. `scryr lint --fix` applies safe fixes. Existing Ruff configuration is honored, and `.scry` files are treated as Python.

## Serve locally

```sh
scryr serve
scryr serve --watch --port 9000
scryr serve --poll 60
scryr serve --no-format --no-open
scryr serve --server-only --sample mern
```

`serve` starts the embedded diagram and GraphQL server, waits for readiness, formats and checks the source, uploads the artifact, and opens the browser.

- `--watch`: repeat after source changes.
- `--poll [seconds]`: poll configured providers every 300 seconds by default; accepts 15–3600 seconds. Bare `--poll` uses 300.
- `--server-only`: do not read, format, execute, or upload a local manifest.
- `--no-format`: verify formatting without changing files.
- `--no-open`: do not launch a browser.
- `--host`: bind interface; default `127.0.0.1`.
- `--port`: TCP port; default `8000`.
- `--sample`: select a persisted sample artifact.
- `--auth-mode`: `local` or `clerk`.

Provider polling starts with an immediate sync after a valid local diagram loads, continues independently of `--watch`, and stops with the server. Only explicit provider sources are collected; a manifest without them does not invoke `gh`. `--server-only` does not start local collection.

## Sync GitHub

```sh
scryr sync github
scryr sync github --manifest api --endpoint http://127.0.0.1:8000/graphql
scryr sync github --path architecture/index.scry --manifest-dir .
```

Fetches the latest ten runs per selected GitHub Actions workflow and complete job snapshots, then attaches observations to stable manifest IDs in a running Scryr server. It uses your existing `gh auth login` authentication. It does not upload the diagram; use `push` or `serve` for that.

Declare `github.repo_url` and `cicd.source=ActionsReportSource(workflows=["ci.yml"], branch="main")` on a block with a stable `manifest_id`. Numeric `workflow_id` remains supported instead of `workflows`. An omitted branch uses the repository default. See [Integrations](/integrations/#poll-github-actions-locally) for a complete declaration.

`--manifest` selects a public variable, name, or stable ID; omitting it syncs all configured blocks. `--endpoint` follows the same precedence and authentication rules as `push`. Missing GitHub CLI, authentication, or API access produces an error without clearing previously collected history.

## Shared path options

These apply to check, format, lint, push, serve, sync, export, inspect, and query:

```text
--path <file>          Manifest file. Defaults to index.scry.
--manifest-dir <dir>   Manifest project root. Defaults to the current directory.
--scryr-dir <dir>      Scryr-managed local state directory.
```

## Push

`scryr push` validates the manifest, generates schema and diagram artifacts, and persists them through the configured GraphQL API.

```sh
scryr push --path index.scry
```

Local endpoints use tokenless local authentication. Hosted Clerk-backed endpoints require `scryr auth login` first.

Endpoint precedence is `--endpoint`, `SCRYR_ENDPOINT`, legacy `SCRYR_GRAPHQL_URL`, then loopback using `PORT` or port 8000.

## Inspect

```sh
scryr inspect types --path index.scry
scryr inspect schema --path index.scry
```

`inspect types` prints declared and runtime field metadata. `inspect schema` prints the Pydantic JSON schema for `Manifest`, `Forge`, and `Diagram`.

## Export

```sh
scryr export mise --forge "Developer environment"
scryr export compose --forge "Developer environment"
scryr export devcontainer --forge "Developer environment"
```

Compose export currently supports PostgreSQL, MongoDB, Redis, MySQL, and MariaDB service tools declared in a `Forge`.

## Authentication

```sh
scryr auth login
scryr auth whoami
scryr auth logout
```

Local use needs no cloud login. Authentication commands manage the OAuth token used for hosted uploads.
