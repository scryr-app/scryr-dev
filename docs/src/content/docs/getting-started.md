---
title: Getting started
description: Install Scryr and turn your first index.scry into an interactive architecture diagram.
---

Choose the path that fits what you want to try:

- **Cloud:** [Log in to Scryr](https://scryr.app), open a sample architecture, and edit it in the hosted app. There is nothing to install.
- **Open source (OSS):** Install the standalone CLI, create a small local diagram without a repository, and then bring Scryr into your own codebase.

The open-source CLI embeds the Python manifest SDK, GraphQL server, and interactive diagram, so Homebrew is the only installation step.

## Open-source quick start

### 1. Install with Homebrew

```sh
brew install scryr-app/tap/scryr
```

### 2. Create a simple first diagram

Create an empty folder and enter it:

```sh
mkdir scryr-quick-start
cd scryr-quick-start
```

Save this small example as `index.scry`:

```python title="index.scry"
from scryr import Diagram, Github, Info, Manifest
from scryr.types import ProgrammingLanguage, WebFramework

api = Manifest(name="API", connections=[Manifest(name="Database")])
database = Manifest(name="Database")

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

### 3. Check and open the diagram

```sh
scryr serve --watch
```

By default, `serve` starts the embedded UI and GraphQL server at `127.0.0.1:8000`, formats and validates the source, uploads the artifact, and opens the browser.
## Use Scryr in your own repository

### 1. Open your repository

```sh
cd /path/to/your/repository
```

### 2. Create the repository manifest

Add `index.scry` at the repository root. You can start by copying the small example above, or [give the Scryr prompt to your coding agent](/llm-prompt/) to generate a model of the repository for you.

Review the generated file before running it. A `.scry` file is Python and can execute imported code.

### 3. Check and serve it

```sh
scryr check
scryr serve --watch
```


## Use another entrypoint

`--path` defaults to `index.scry`, and `--manifest-dir` defaults to the current directory.

```sh
scryr check --path architecture/platform.scry --manifest-dir .
scryr serve --path services/catalog/index.scry --manifest-dir .
```

Nested entrypoints are grouped by their parent folder in the diagram selector.

## Next steps

- [Understand manifests, diagrams, queries, and forges](/manifests/)
- [Generate a repository manifest with your LLM](/llm-prompt/)
- [Enable `.scry` highlighting](/editors/)
