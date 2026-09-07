# Contributing

Scryr has a Python SDK and samples in `manifest`, a Rust CLI/server in `crystal`,
and a Vite frontend in `map`. All terminal, VS Code, CI, release, and deployment
commands and their implementations live in [mise.toml](mise.toml). Use the
canonical task names below; there are no task aliases or external task scripts.

## Setup

Install [mise](https://mise.jdx.dev/getting-started.html), then run from the root:

```bash
mise install
mise run setup
```

The checked-in lockfiles select tool and dependency versions. Deployment and
publishing tools are installed on demand by their tasks. Put personal cloud
configuration in ignored `mise.local.toml`; copy
[mise.local.toml.example](mise.local.toml.example) as a starting point.

## Development and deployment modes

| Mode | Command | Behavior |
| --- | --- | --- |
| Contribute OSS | `mise run dev:oss` | Rust restarts on changes; Vite HMR; local SQLite and local auth |
| Contribute cloud | `mise run dev:cloud` | Rust restarts and Vite HMR locally; remote Turso and Clerk |
| Frontend against Fly | `mise run dev:cloud:remote` | Vite HMR using `CLOUD_GRAPHQL_URL` |
| Build standalone OSS | `mise run build:oss` | Release CLI embeds Vite, Python SDK, and sample artifacts |
| Distribute OSS | `mise run distribute:oss` | Publish npm packages, a Homebrew formula, and crates.io packages |
| Deploy cloud | `mise run deploy:cloud` | Build, migrate Turso, deploy Fly, then deploy Vercel |

### Contribute OSS

`dev:oss` starts both processes. Open `http://127.0.0.1:3000`; the API runs at
`http://127.0.0.1:8000/graphql`. Rust restarts when its source or the SDK changes;
Vite handles frontend hot reload. Ctrl+C stops both. Run `dev:server:oss` or
`dev:ui:oss` for just one process. `run:server:local` runs the server once for
tasks that manage its lifetime.

OSS mode explicitly removes cloud database/auth variables and selects local
auth, even when `mise.local.toml` contains cloud configuration. SQLite and runtime
state live in `.cache/oss`. Local auth grants a shared writable identity; the
contributor server binds to loopback.

```bash
mise run codegen-graphql-requests
mise run run:cli:samples
```

These tasks reuse the local server or start one temporarily.

### Contribute cloud

Set `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `CLERK_SECRET_KEY`, and
`VITE_CLERK_PUBLISHABLE_KEY`. Use a development database and Clerk instance.
`dev:cloud` keeps both application processes local for reload and debugging while
using the remote database. It selects Clerk auth for both frontend and server.
Configure Clerk to allow `http://127.0.0.1:3000`.

Set `CLOUD_GRAPHQL_URL` to an HTTPS Fly `/graphql` endpoint to run
`dev:cloud:remote`. Allow the local Vite origin in the Fly app's CORS configuration.
This mode is useful for comparing local frontend behavior with the hosted Vercel
frontend. It does not start another local Rust server.

### VS Code debugging

Open the repository folder or either checked-in `.code-workspace` file. Install
the recommended Rust Analyzer, CodeLLDB, Python, and Biome extensions. Ensure
`mise` is on the PATH inherited by VS Code.

- Run **Debug OSS** or **Debug CLOUD** to launch Rust under CodeLLDB and a browser
  against Vite. The prelaunch tasks build Rust with debug symbols and generate
  a private `.cache/debug-<mode>.env` from the same runtime configuration as mise.
- Rust breakpoints work in the local server in both modes, including calls to
  Turso in cloud mode. Restart the debug session after Rust edits. Vite continues
  to hot reload during browser debugging.
- **Rust: attach to running server** attaches to a watcher-started process. A
  Rust restart changes the PID, so attach again after that restart.
- **Browser: hosted Vercel** opens a deployment URL for browser inspection.
  Source breakpoints in a hosted build require source maps from that build.

Stop a running `dev:*` stack before launching a debug compound to free ports
8000 and 3000. The debugging configurations use open-source CodeLLDB and VS
Code's built-in JavaScript debugger. No paid debugging service is required.

## Build and install the standalone CLI

```bash
mise run build:oss
mise run release:smoke
mise run install:oss
```

The output is `crystal/target/release/scryr`. Its HTTP server serves the embedded
Vite UI with a relative `/graphql` URL and local auth. It also embeds the Python
SDK and generated sample architectures. No checkout, Node installation, or cloud
account is required at runtime. First manifest execution downloads Scryr-managed
uv/Python and Python dependencies; this is not an entirely offline Python runtime.

`install:oss` copies the binary to `${SCRYR_CLI_INSTALL_DIR:-$HOME/.local/bin}`
and configures supported shell profiles. `release:smoke` exercises the embedded
SDK outside the checkout, the HTTP UI, readiness, and a GraphQL upload.
`build:docker` builds the prepared OSS binary into a local server image.

## Cloud deployment

Create the Turso database, Fly app, Clerk instance, and Vercel project first.
Configure the Vercel project with `map` as its project root and a Node version
compatible with `map/package.json`. Resource creation and billing setup are
account administration steps; deployment updates the configured resources.

Supply the values shown in `mise.local.toml.example`. In addition to the cloud
development credentials, deployment requires `FLY_API_TOKEN`, `FLY_APP`,
`CLOUD_GRAPHQL_URL`, `CORS_ALLOWED_ORIGINS`, `VERCEL_TOKEN`, `VERCEL_ORG_ID`, and
`VERCEL_PROJECT_ID`. CORS must include the exact Vercel frontend origin.

```bash
mise run deploy:cloud
```

The task validates configuration, builds the CLI and cloud UI, runs `scryr
migrate` against Turso, stages Fly secrets, deploys from the `crystal` Docker
context, and builds/publishes Vercel production output. The Docker Rust version
comes from mise. Fly explicitly uses Clerk auth; the Vercel UI targets the public
Fly GraphQL URL. Fly also embeds a Clerk-enabled UI with a relative API endpoint; the primary
hosted frontend is served from Vercel.

`deploy:turso`, `deploy:fly`, `deploy:vercel`, and `deploy:secrets` are available
for targeted maintenance. Build first with `build:cloud` before standalone
`deploy:fly`. Database migration reuses the server's schema bootstrap and upgrade
logic; it does not create databases or seed sample data. A failure stops subsequent
deployment steps. Deployments across providers are sequential, not atomic.

See [repository maintenance](docs/maintenance.md) for GitHub environments,
publishing credentials, and the tag-to-release process.

## Checks and pull requests

```bash
mise run static       # formatting, lint, types
mise run test         # unit tests, then integration tests
mise run fix          # format and apply lint fixes
mise run ci:quality   # automation checks and secret scan
```

Narrow commands include `lint:rust`, `format:check:python`, `type-check:typescript`,
`test:unit:rust`, and `test:integration:python`. GitHub runs `ci:python`, `ci:rust`,
`ci:ui`, and `ci:release`; the same tasks work locally. The CLI tests need network
access when provisioning Python for the first time.

Open focused pull requests against `main`, with relevant tests and checks passing.
Run `build:oss` and `release:smoke` for embedded asset or packaging changes.
Regenerate GraphQL types with `codegen-graphql-requests` and the crate-local SDK
with `sync:embedded-sdk`. Do not commit generated build churn unless intentionally
updating the bundled assets. Include version, platform, reproduction steps, and
relevant logs in bug reports.
