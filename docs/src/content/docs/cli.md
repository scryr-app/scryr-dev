---
title: CLI reference
description: Check, format, serve, push, inspect, export, and collect local project evidence.
---

Run commands from the repository containing `index.scry`.

## Primary workflows

```sh
scryr check
scryr format
scryr lint --fix
scryr push
scryr serve --watch
scryr collect doctor
scryr collect list
scryr collect run --manifest services/api --section tests --collector unit
scryr collect status
```

## Check, format, and lint

### `scryr check`

Checks formatting, lint, Python types, manifest execution, serialization, and Scryr diagram rules. It requires at least one public `Diagram`, unique diagram identifiers, and valid references to public `Manifest` objects.

`check` does not rewrite source or contact Scryr APIs. Manifest execution itself runs user Python code and can have whatever side effects that code defines.

### `scryr format`

Rewrites the entrypoint and reachable local `.scry`/`.py` imports. Use `scryr format --check` for a non-writing check.

### `scryr lint`

Reports lint errors. `scryr lint --fix` applies safe fixes. Existing Ruff configuration is honored, and `.scry` files are treated as Python.

## Serve locally

```sh
scryr serve
scryr serve --watch --port 9000
scryr serve --no-format --no-open
scryr serve --server-only --sample mern
```

`serve` starts the embedded diagram and GraphQL server, waits for readiness, formats and checks the source, uploads the artifact, and opens the browser. After valid local publication it starts the declared collector schedules; `--watch` controls source reload only.

- `--watch`: repeat after source changes.
- `--server-only`: do not read, format, execute, or upload a local manifest; never register collectors.
- `--no-collect`: serve the map with collection paused.
- `--no-format`: verify formatting without changing files.
- `--no-open`: do not launch a browser.
- `--host`: bind interface; default `127.0.0.1`.
- `--port`: TCP port; default `8000`.
- `--sample`: select a persisted sample artifact.
- `--auth-mode`: `local` or `clerk`.

## Shared path options

These apply to check, format, lint, push, serve, export, inspect, and collect:

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

## Collect local evidence

`collect` uses the concrete integration declarations in the six section lists.
`doctor` explains tool/version/auth availability, `list` shows effective declarations,
`run` executes the selected manifest/section/collector, and `status` shows attempts
and freshness. Manual runs use the active local owner when available or claim the
workspace lease themselves. Tests/checks/benchmarks are manual unless explicitly
scheduled. Watching a report file never starts its producer.

Tool installation is explicit; server startup does not install scanners. Reads of
GraphQL or the cards never execute tools. Only locally selected trusted source can
register execution. Source constructor/preview evaluation does not run collectors.

## Authentication

```sh
scryr auth login
scryr auth whoami
scryr auth logout
```

Local use needs no cloud login. Authentication commands manage the OAuth token used for hosted uploads.
