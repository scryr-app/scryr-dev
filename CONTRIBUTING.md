# Contributing

Scryr's mise tasks follow four workflows: `contribute:*`, `verify:*`,
`release:*`, and `deploy:*`. Component selectors match repository directories:
`manifest` is the Python SDK and samples, `crystal` is the Rust CLI/server, and
`map` is the Vite frontend.

All task implementations live in [mise.toml](mise.toml). There are no aliases or
external task scripts. Run commands from the repository root. `mise tasks ls`
lists public commands; `internal:*` helpers are hidden from that list.

## Contribute

Install [mise](https://mise.jdx.dev/getting-started.html), then:

```bash
mise install
mise run contribute:setup
mise run contribute:oss
```

Open `http://127.0.0.1:3000`; the API runs at
`http://127.0.0.1:8000/graphql`. Rust restarts on source or SDK changes, and Vite
provides hot reload. Ctrl+C stops both processes. After making changes, run:

```bash
mise run pre-commit
```

`pre-commit` runs `contribute:fix` followed by `verify`. It applies formatting and
lint fixes to source files, then runs the full CI checks. Review the resulting
diff before committing. This is a command to run, not an automatically installed
Git hook. Run `mise run contribute` to show this workflow in the terminal.

The checked-in lockfiles select tool and dependency versions. Deployment and
publishing tools are installed on demand by their tasks. Put personal cloud
configuration in ignored `mise.local.toml`; copy
[mise.local.toml.example](mise.local.toml.example) as a starting point.

### Development modes

| Command | Behavior | Prerequisites |
| --- | --- | --- |
| `contribute:oss` | Rust reload and Vite HMR with SQLite and local auth | Contributor setup |
| `contribute:cloud` | Rust reload and Vite HMR locally with remote Turso and Clerk | Development cloud credentials |
| `contribute:cloud:remote-map` | Local Vite HMR against the deployed Fly API | Cloud credentials and `CLOUD_GRAPHQL_URL` |
| `contribute:oss:server` / `contribute:cloud:server` | Watch and restart only the server | Corresponding mode's setup |
| `contribute:oss:map` / `contribute:cloud:map` | Run only Vite HMR | Corresponding mode's setup |

OSS mode removes cloud database/auth variables and selects local auth even when
`mise.local.toml` contains cloud configuration. SQLite and runtime state live in
`.cache/oss`. Local auth grants a shared writable identity; the contributor server
binds to loopback.

Cloud mode requires `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `CLERK_SECRET_KEY`,
and `VITE_CLERK_PUBLISHABLE_KEY`. Use a development database and Clerk instance.
Both application processes run locally; the database is remote. Configure Clerk
to allow `http://127.0.0.1:3000`.

For `contribute:cloud:remote-map`, set `CLOUD_GRAPHQL_URL` to an HTTPS Fly
`/graphql` endpoint and allow the local Vite origin in Fly's CORS configuration.
This mode does not start a local Rust server.

### Generation and builds

```bash
mise run contribute:generate:graphql        # Generate frontend GraphQL types
mise run contribute:generate:manifest-types # Generate manifest field metadata
mise run contribute:seed:samples            # Upload sample manifests locally
mise run contribute:build                   # Build all development artifacts
```

GraphQL generation and sample upload reuse the local server or start one
temporarily. `contribute:build:manifest`, `contribute:build:crystal`, and
`contribute:build:map` build individual components after contributor setup.
Development builds do not prepare the standalone CLI's embedded UI; use
`release:build` for that.

### VS Code debugging

Open the repository folder or either checked-in `.code-workspace` file. Install
the recommended Rust Analyzer, CodeLLDB, Python, and Biome extensions. Ensure
`mise` is on the PATH inherited by VS Code.

- **Debug OSS** and **Debug CLOUD** launch Rust under CodeLLDB and a browser
  against Vite. The checked-in editor tasks call `contribute:oss:debug` or
  `contribute:cloud:debug`, which build Rust with symbols and write a private
  `.cache/debug-<mode>.env`. The corresponding `contribute:<mode>:map` task
  starts Vite and waits for it to be ready.
- Rust breakpoints work in both local server modes, including calls to Turso.
  Restart the debug session after Rust edits. Vite continues to hot reload.
- **Rust: attach to running server** attaches to a watcher-started process. A
  restart changes the PID, so attach again afterward.
- **Browser: hosted Vercel** opens a deployment URL. Source breakpoints in a
  hosted build require source maps from that build.

Stop a running contribution stack before launching a debug compound to free
ports 8000 and 3000. Debugging uses open-source CodeLLDB and VS Code's built-in
JavaScript debugger.

## Verify before a pull request

```bash
mise run verify       # All CI checks, including standalone release smoke tests
mise run pre-commit   # Apply fixes, then run the same complete verification
```

`verify` sequentially runs `verify:automation`, `verify:manifest`,
`verify:crystal`, `verify:map`, and `verify:release`. GitHub Actions calls those
same tasks in separate jobs. Component suites prepare their dependencies;
verification does not apply source fixes or publish anything. Builds can refresh
generated assets. The automation suite checks workflow policy, runs automation
regressions, validates workflow syntax, and scans Git history for secrets.

For faster feedback after `contribute:setup`:

| Command | Scope |
| --- | --- |
| `verify:static` | Formatting checks, lint, and types across components |
| `verify:test` | Unit tests, then integration tests |
| `verify:manifest` / `verify:crystal` / `verify:map` | One component's complete CI suite |
| `verify:lint:crystal` | Rust Clippy |
| `verify:format:manifest` | Python formatting checks |
| `verify:types:map` | TypeScript type checks |
| `verify:test:unit:crystal` | Rust unit tests |
| `verify:test:integration:manifest` | Python sample integration tests |
| `verify:workflows` | Mise-only workflow and local composite-action policy |
| `verify:release` | Build and smoke-test the standalone CLI |

Use `contribute:fix` to apply formatting and lint fixes without verification.
CLI tests need network access when provisioning managed Python for the first time.
Automation checks provision pinned PyYAML through uv without installing the SDK.

Open focused pull requests against `main`. Do not commit generated build churn
unless intentionally updating bundled assets. Release builds and crystal checks
refresh the embedded manifest SDK automatically. Include version, platform,
reproduction steps, and relevant logs in bug reports.

### GitHub Actions contract

Every workflow `run` step must be exactly `mise run <declared-task>`. Both `.yml`
and `.yaml` files are checked structurally, including quoted commands, YAML
blocks, local reusable workflows, and local composite actions. Unknown tasks,
additional shell commands, dynamic task names, and custom shell command wrappers
are rejected. Local actions must be composites whose commands follow this rule.

Workflow YAML owns triggers, permissions, environments, matrices, and artifact
transfer. The approved external actions are checkout, mise installation, artifact
upload, and artifact download; changes to that list require updating the policy
in mise. Dependency-graph submission calls
`internal:github:submit-dependencies`; it writes to GitHub and is not part of
local verification.

## Release the standalone CLI

Run `mise run release` for the command guide. Building locally, creating a GitHub
draft, and distributing a published release are separate stages.

### Build and install locally

```bash
mise run release:build
mise run verify:release:smoke
mise run contribute:install
```

The output is `crystal/target/release/scryr`. It embeds the Vite UI, Python SDK,
and generated sample architectures. Its HTTP server uses a relative `/graphql`
URL and local auth. No checkout, Node installation, or cloud account is required
at runtime. First manifest execution downloads Scryr-managed uv/Python and Python
dependencies; it is not an entirely offline Python runtime.

`verify:release:smoke` requires an existing release build and exercises the SDK
outside the checkout, the HTTP UI, readiness, and a GraphQL upload.
`verify:release` performs both the build and smoke test.
`contribute:install` builds the CLI, copies it to
`${SCRYR_CLI_INSTALL_DIR:-$HOME/.local/bin}`, and configures supported shell
profiles. `release:build:docker` builds a local server image and needs Docker.

### Create the GitHub release

1. Update versions consistently in the crystal crate `Cargo.toml` files,
   manifest project `pyproject.toml` files, and map's `package.json` and
   `package-lock.json`. Refresh affected dependency lockfiles and merge the
   version change into `main` with verification passing.
2. Create and push the corresponding `vX.Y.Z` tag on that commit. The tag-triggered
   Release workflow runs `release:validate`, the shared CI checks, and then
   `verify:release` plus `release:package` on Linux and macOS, each for x86_64
   and ARM64.
3. GitHub Actions collects the four archives and checksums in `dist/release`
   and calls `release:draft`. Review its notes and artifacts, then publish the
   stable draft on GitHub.

For local stage execution, check out the exact tag and set `RELEASE_TAG=vX.Y.Z`.
`release:validate` requires the tag, `HEAD`, `origin/main`, and package versions
to agree, with the tagged commit on `main`. Fetch tags and main history first.
`release:package` archives the current machine's tested binary; it does not
cross-compile. `release:draft` requires all four platform archives, matching
checksums, and GitHub authentication (`GH_TOKEN` and `GH_REPO`).

### Distribute the published release

Publishing a stable GitHub Release triggers the Distribute OSS workflow. For a
manual run, choose its published tag. It calls `release:validate` followed by
`release:distribute` from that tag's checkout, using the `publish` environment.

To execute those same stages locally from the tag's checkout:

```bash
export RELEASE_TAG=vX.Y.Z # Replace with the actual published stable tag
mise run release:validate
mise run release:distribute
```

Set `GH_REPO`, GitHub authentication, `NPM_PACKAGE_NAME`, `NPM_TOKEN`,
`HOMEBREW_TAP`, `HOMEBREW_TAP_TOKEN`, and `CARGO_REGISTRY_TOKEN` as shown in
`mise.local.toml.example`. Distribution validates settings, downloads and verifies
the published artifacts, prepares packages, and publishes npm, Homebrew, then
crates.io in order. A draft or prerelease is not eligible.

`release:distribution:prepare` prepares publisher inputs without publishing.
The targeted `release:distribute:npm`, `release:distribute:homebrew`, and
`release:distribute:crates` tasks are maintenance operations requiring their
respective credentials and prepared inputs. Publishing across registries is
sequential, not atomic.

## Deploy cloud

Run `mise run deploy` for the command guide. Create the Turso database, Fly app,
Clerk instance, and Vercel project first. Configure Vercel with `map` as its
project root and a Node version compatible with `map/package.json`. Deployment
updates configured resources; resource creation and billing are account setup.

Supply the values in `mise.local.toml.example`. Beyond cloud development
credentials, deployment requires `FLY_API_TOKEN`, `FLY_APP`, `CLOUD_GRAPHQL_URL`,
`CORS_ALLOWED_ORIGINS`, `VERCEL_TOKEN`, `VERCEL_ORG_ID`, and `VERCEL_PROJECT_ID`.
CORS must include the exact Vercel frontend origin.

In GitHub Actions, run **Deploy cloud** with the versioned release tag. The
workflow checks out and validates that tag, then deploys using the `production`
environment. To run the same stages locally, check out the tag, fetch main
history, configure credentials, and execute:

```bash
export RELEASE_TAG=vX.Y.Z # Replace with the actual release tag
mise run release:validate
mise run deploy:cloud
```

The task checks configuration, builds the CLI and cloud UI, runs `scryr migrate`
against Turso, stages Fly secrets, deploys from the `crystal` Docker context,
and builds/publishes Vercel production output. The Docker Rust version comes
from mise. Fly uses Clerk auth and embeds a Clerk-enabled UI with a relative API
endpoint; the primary frontend is served by Vercel and targets the public Fly API.

`deploy:database:migrate`, `deploy:fly`, `deploy:vercel`, and `deploy:secrets`
allow targeted maintenance. Run `deploy:build` before standalone `deploy:fly`.
Migration reuses the server's schema upgrade logic; it does not create databases
or seed sample data. Failure stops subsequent steps. Provider deployments are
sequential, not atomic.
