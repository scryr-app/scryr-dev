# 🔮 Scryr

Define architecture and local development evidence in a typed Python `index.scry`,
explore it in a React/Three.js map, and run collection with the Rust `scryr` CLI.

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
its diagrams automatically. It also schedules the collectors declared in that
file. Add `--watch` to reload after source edits, or `--no-collect` to start with
collection paused. `--server-only` serves existing uploaded maps without
reading local manifests or running collectors.
This workflow needs no Clerk, Turso, Fly, or other cloud account. The build and first manifest
execution need internet access to download dependencies and managed Python.

Local mode grants requests a shared writable identity. Keep it bound to loopback.
Only run trusted `.scry` files: manifests execute Python code.

## Local development cards

Put concrete SDK collectors directly in the sections of each `Manifest`:

```python
from scryr import (
    Diagram, GitStatusCollector, Manifest, PytestCollector, RuffCheckCollector,
)

api = Manifest(
    manifest_id="api",
    name="API",
    repository=[GitStatusCollector()],
    checks=[RuffCheckCollector(paths=["src"])],
    tests=[PytestCollector(paths=["tests"])],
)
system = Diagram(name="Local development", manifests=[api])
```

The six sections are `repository`, `checks`, `metrics`, `tests`, `dependencies`,
and `performance`. GitHub pull requests and remote Actions runs belong in
`repository`; local lint/build tasks belong in `checks`. OpenMetrics endpoints
and Docker provide laptop metrics. Pytest, Vitest, Nextest, and existing JUnit or
coverage files provide test evidence. Dependencies combines Syft inventory,
Grant-backed license policy, and Grype vulnerabilities. Hyperfine supplies
explicitly requested benchmarks.

```bash
scryr collect list
scryr collect doctor
scryr serve --watch
# In another terminal in the same project:
scryr collect run --manifest api --section tests
scryr collect status
scryr collect pause
scryr collect resume
```

Install the collector tools you choose in your project environment or `PATH`;
Scryr does not install them automatically. Constructors only describe settings.
Git and lightweight metrics can poll; tests, checks, and benchmarks default to
manual runs. Schedules, tool requirements, environment references, and dependency
policy are all declared in `index.scry`.

Observations are stored in local SQLite (`.scryr/scryr.db` by default) and exposed
through the local GraphQL API. Collector state and temporary artifacts live under
`.scryr/collection`. Cards show collection time, source age, and stale or missing
evidence; importing an old report does not make its results current. See the
[CLI guide](crystal/crystal-cli/README.md) and
[typed integration examples](docs/src/content/docs/integrations.md).

To generate an artifact from a bundled sample:

```bash
./crystal/target/release/scryr inspect schema \
  --path tests/samples/mern/index.scry --manifest-dir manifest
```

Release automation builds Linux and macOS archives for x86_64 and ARM64, with
SHA-256 checksums, as draft GitHub Releases. Maintainers review and publish them;
see available builds on the [releases page](https://github.com/scryr-app/scryr-dev/releases).
The distribution workflow publishes the CLI through Homebrew.

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
