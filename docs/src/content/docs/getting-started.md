---
title: Getting started
description: Install Scryr and turn your first index.scry into an interactive architecture map.
---

Scryr combines a typed Python manifest SDK, a Rust CLI and GraphQL server, and a React/Three.js map. The standalone CLI embeds the map and SDK, so one install is enough for the normal local workflow.

## Install

### Homebrew

```sh
brew install scryr-app/tap/scryr
```

### npm

```sh
npm install --global @scryr/cli
```

### Cargo

```sh
cargo install crystal-cli
```

Native release archives for macOS and Linux on ARM64 and x86_64 are also available from [GitHub Releases](https://github.com/scryr-app/scryr-dev/releases).

```sh title="Verify the installation"
scryr --version
scryr --help
```

Scryr is pre-1.0. Keep the CLI and the manifests in a project versioned together.

## Create `index.scry`

Save this file at the root of your repository:

```python title="index.scry"
from scryr import Diagram, Github, Info, Manifest
from scryr.types import ProgrammingLanguage, WebFramework

web = Manifest(
    manifest_id="apps/web",
    name="Web App",
    connections=[Manifest(name="Public API")],
    info=Info(
        description="Customer-facing application",
        language=ProgrammingLanguage.typescript,
        frameworks=[WebFramework.react],
        owner_team="Product",
    ),
)

api = Manifest(
    manifest_id="services/api",
    name="Public API",
    connections=[Manifest(name="Postgres")],
    info=Info(
        description="Business API",
        language=ProgrammingLanguage.python,
        frameworks=[WebFramework.fastapi],
        owner_team="Platform",
    ),
    github=Github(repo_url="https://github.com/acme/api"),
)

database = Manifest(name="Postgres")
architecture = Diagram(name="System architecture", manifests=[web, api, database])
```

Connection targets are resolved by manifest name. The reference in `Manifest(name="Public API")` must match the real component name exactly.

## Check and open the map

```sh
scryr check
scryr serve --watch
```

By default, `serve` starts the embedded UI and GraphQL server at `127.0.0.1:8000`, formats and validates the source, uploads the artifact, and opens the browser. Watch mode keeps the previous valid diagram visible when a later edit fails.

On first execution, Scryr provisions its own pinned `uv`, managed Python runtime, and per-project environment. It does not change your system Python.

:::tip[Try the hosted map]
Open [scryr.app](https://scryr.app) to explore the cloud experience. Cloud editing stores source snapshots; local `scryr serve` can save the registered entrypoint back to disk.
:::

## Use another entrypoint

`--path` defaults to `index.scry`, and `--manifest-dir` defaults to the current directory.

```sh
scryr check --path architecture/platform.scry --manifest-dir .
scryr serve --path services/catalog/index.scry --manifest-dir .
```

Nested entrypoints are grouped by their parent folder in the map selector.

## Next steps

- [Understand manifests, diagrams, queries, and forges](/manifests/)
- [Generate a repository manifest with your LLM](/llm-prompt/)
- [Enable `.scry` highlighting](/editors/)
