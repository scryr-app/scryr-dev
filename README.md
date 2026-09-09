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
mise install rust python uv node
mise run install-dependencies
mise run build:oss
./crystal/target/release/scryr serve
```

Open `http://127.0.0.1:8000`. The binary embeds the map UI and Python SDK.
Local startup serves only uploaded maps; it does not seed bundled samples.
This workflow needs no Clerk, Turso, Fly, or other cloud account. The build and first manifest
execution need internet access to download dependencies and managed Python.

Local mode grants requests a shared writable identity. Keep it bound to loopback.
Only run trusted `.scry` files: manifests execute Python code.

To generate an artifact from a bundled sample:

```bash
./crystal/target/release/scryr generate schema \
  --path samples/mern/index.scry --manifest-dir manifest
```

Release automation builds Linux and macOS archives for x86_64 and ARM64, with
SHA-256 checksums, as draft GitHub Releases. Maintainers review and publish them;
see available builds on the [releases page](https://github.com/scryr-app/scryr-dev/releases).
The distribution workflow supports npm, Homebrew, and crates.io.

## Develop and contribute

- `manifest`: Python SDK, `.scry` examples, and Python tests.
- `crystal`: Rust GraphQL server, SQLite/Turso persistence, and CLI.
- `map`: React/Three.js frontend.

```bash
mise run dev:oss # Rust reload + Vite HMR, local SQLite
# mise run dev:cloud # local reload + remote Turso/Clerk
mise run static
mise run test
```

Read [CONTRIBUTING.md](CONTRIBUTING.md) for local development and pull requests,
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community expectations, and
[repository maintenance](docs/maintenance.md) for CI, security notices, and releases.
Report bugs through the [issue templates](https://github.com/scryr-app/scryr-dev/issues/new/choose).
Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).
See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for redistributed materials.
