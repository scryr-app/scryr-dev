# 🔮 Scryr

Actionable architecture. Define architecture in Python `.scry` manifests, explore
it in a React/Three.js map, and generate artifacts with the Rust `scryr` CLI.

[![CI](https://github.com/scryr-app/scryr-dev/actions/workflows/ci.yml/badge.svg)](https://github.com/scryr-app/scryr-dev/actions/workflows/ci.yml)

Scryr is pre-1.0 software licensed under the [MIT License](LICENSE).

## Build and run from source

Install [mise](https://mise.jdx.dev/getting-started.html) and Git, then:

```bash
git clone https://github.com/scryr-app/scryr-dev.git
cd scryr-dev
mise install
mise run contribute:setup
mise run release:build
./crystal/target/release/scryr serve --server-only
```

Open `http://127.0.0.1:8000`. The binary embeds the map UI and Python SDK.
Use `scryr serve` from a repository containing `index.scry` to format, check, and load
its diagrams automatically. `--server-only` serves existing uploaded maps without
reading or modifying local manifests. Add `--watch` to reload after source edits.
This workflow needs no Clerk, Turso, Fly, or other cloud account. The build and first manifest
execution need internet access to download dependencies and managed Python.

Local mode grants requests a shared writable identity. Keep it bound to loopback.
Only run trusted `.scry` files: manifests execute Python code.

To generate an artifact from a bundled sample:

```bash
./crystal/target/release/scryr inspect schema \
  --path tests/samples/mern/index.scry --manifest-dir manifest
```

Release automation builds Linux and macOS archives for x86_64 and ARM64, with
SHA-256 checksums, as draft GitHub Releases. Maintainers review and publish them;
see available builds on the [releases page](https://github.com/scryr-app/scryr-dev/releases).
The distribution workflow publishes the CLI through Homebrew.

## Local collectors and observation cards

Define typed command tasks, interval collectors, and cards in `index.scry`.
Python compiles the definitions and their output schemas; Rust executes commands,
validates JSON output, and stores observations. Multiple cards referencing one
collector share its results. See the [SDK example](manifest/scryr/README.md) and
[runnable collector sample](manifest/tests/samples/collectors/index.scry).

```bash
# Run every collector once (or add --collector <id>).
scryr collect

# Run interval collectors and view their cards in the local map UI.
scryr serve --collect

# View previously stored observations without executing collector commands.
scryr serve
```

The sample can be run from this checkout with
`scryr serve --collect --manifest-dir manifest --path tests/samples/collectors/index.scry`.
Open **Observations** in the map to see cards for the selected diagram's components.
The sample reports fixture data; it does not contact a production service.

Collector commands run with your local account's permissions and inherited
environment. Only execute trusted definitions. Commands use argument arrays,
closed stdin, explicit timeouts, and bounded output; a shell is used only if you
explicitly name one in the arguments. Working directories resolve relative to the
manifest entrypoint's directory. This initial runner supports Unix platforms.

Observations persist in `.scryr/collectors.sqlite` beside the manifest entrypoint.
Opening a card, previewing source, uploading a manifest, and saving editor changes
do not execute collector commands. As with all `.scry` files, evaluating the
manifest itself executes Python. The execution plan is pinned when the server
starts; restart to activate changed definitions. `--collect` cannot be combined
with `--watch`, `--server-only`, or hosted authentication.

This is the local collection milestone: JSON stdout, whole-second intervals,
generic status/table/history cards, and read-only polling. Remote runners,
credential references, cron schedules, interactive tasks, typed projections,
specialized deployment cards, and automatic reload of executable plans are not
implemented. Existing manifests without collectors keep their existing format.
The local scheduler runs collectors sequentially on fixed intervals and skips
missed runs. A process lease prevents competing
runners for the same observation store; viewing stored observations remains available.

## Develop and contribute

- `manifest`: Python SDK, `.scry` examples, and Python tests.
- `crystal`: Rust GraphQL server, SQLite/Turso persistence, and CLI.
- `map`: React/Three.js frontend.

```bash
mise run contribute:setup
mise run contribute:oss # Rust reload + Vite HMR, local SQLite
mise run pre-commit     # Apply fixes, then run all CI checks
```

Use `mise run verify` for the full checks without applying fixes. Component
checks are `verify:manifest`, `verify:crystal`, and `verify:map`.
`mise run release` and `mise run deploy` show the maintainer workflows and inputs.

Read [CONTRIBUTING.md](CONTRIBUTING.md) for local development and pull requests,
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community expectations.
Report bugs through the [issue templates](https://github.com/scryr-app/scryr-dev/issues/new/choose).
Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).
See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for redistributed materials.
