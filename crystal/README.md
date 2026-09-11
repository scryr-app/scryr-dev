# 💎 Crystal

Rust CLI and GraphQL server for Scryr.

## What It Does

- serves the embedded map UI and block data through `scryr serve`
- exposes the GraphQL API used by the UI
- generates bundled manifest artifacts for Docker/runtime use

## Crates

- `crystal-core`: canonical GraphQL models and types, pure artifact generation, and persistence
- `crystal-server`: Actix HTTP server, GraphQL request handling and root resolvers, and authentication
- `crystal-cli`: command parsing and orchestration, Python/Sprite execution, and the `scryr` binary

Dependencies flow from CLI to server/core, and from server to core. Core uses
`async-graphql` to define the canonical object, input, and enum types in
`graphql_types` and `manifest`. The CLI and server use those same definitions:
upload models also derive Serde, and persistence returns the shared `ScryrMap`
GraphQL model directly. There are no separate server DTOs or model conversions.

Core's `generation` module exposes renderers and map splitting; `persistence`
groups storage connections, reads, and writes. `generated_manifest_envelope`
holds the flexible JSON format exchanged with the Python adapter. HTTP handling,
authentication, schema wiring, and root resolvers stay in the server. Generation
and persistence entry points return `crystal_core::Error` with validation or
storage categories. Command and transport boundaries format errors for users.

Dependency versions and lint policy are declared once in the workspace
`Cargo.toml` and inherited by each crate. Cargo package names match directory
names; the CLI executable remains `scryr`:

```bash
cargo run -p crystal-cli --bin scryr -- --help
```

## Database

The server uses SQLite by default for local development. If `DATABASE_URL` is
unset, Scryr creates a SQLite database at `SCRYR_SQLITE_PATH`, or at
`.scryr/scryr.db` when that variable is unset.

Use a SQLite URL when you want to override the local database file:

```bash
# explicit SQLite file
DATABASE_URL=sqlite://./.scryr/scryr.db?mode=rwc
```

Use Turso/libSQL for hosted storage:

```bash
TURSO_DATABASE_URL=libsql://your-database.turso.io
TURSO_AUTH_TOKEN=...
```

## Authentication

Scryr uses zero-config local auth by default. It injects a synthetic local
principal with user id `local-dev-user` and organization id `local-dev-org`, so
GraphQL requests do not need a bearer token.

Hosted/cloud deployments should use Clerk:

```bash
AUTH_MODE=clerk
CLERK_SECRET_KEY=sk_...
```

Set `AUTH_MODE=clerk` locally when testing the hosted authentication path.
When serving local auth on a non-loopback interface such as `0.0.0.0`, treat the
process as unauthenticated on your network unless you put it behind another
access-control layer.

## Common Tasks

```bash
# install toolchain and fetch dependencies
mise install
mise run contribute:setup

# build the standalone scryr binary with embedded map UI
mise run release:build

# build it and update the global scryr command on PATH
mise run contribute:install

# run the distributed-style binary
./target/release/scryr serve

# checks
mise run verify:lint:crystal
mise run verify:types:crystal
mise run verify:test:crystal
```

## Endpoints

- Map UI: `GET http://127.0.0.1:8000/`
- GraphQL HTTP: `POST http://127.0.0.1:8000/graphql`
- Playground: open `http://127.0.0.1:8000/graphql` or `http://127.0.0.1:8000/playground` in a browser
- GraphQL WebSocket: `GET http://127.0.0.1:8000/graphql` with WebSocket upgrade headers
- Liveness: `GET http://127.0.0.1:8000/health`
- Readiness: `GET http://127.0.0.1:8000/ready`

The distributed `scryr` binary serves the Vite build from embedded Rust assets.
Build it with `mise run release:build`; direct `cargo build --release` does not
prepare fresh frontend assets. Packaged crates include the prepared assets under
`crystal-server/static`.

When running with `AUTH_MODE=clerk`, GraphQL requests require a Clerk bearer
token in the `Authorization: Bearer ...` header.

For `scryr` against a local endpoint, upload commands do not need a Clerk login.
For `scryr` against a hosted Clerk-backed endpoint, authenticate first with
`cargo run -p crystal-cli -- auth login`, then run upload commands against
`SCRYR_GRAPHQL_URL`.

## Scryr CLI

```bash
cargo run -p crystal-cli -- serve --sample mern
cargo run -p crystal-cli -- inspect types --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest
cargo run -p crystal-cli -- export compose --path ../manifest/tests/samples/mern/index.scry --manifest-dir ../manifest --forge "MERN Forge"
cargo run -p crystal-cli -- auth whoami
```


Fly deployment health checks use `/ready`, while `/health` remains a lightweight
process liveness endpoint.
