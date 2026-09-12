---
title: Manifest language
description: Model components, connections, diagrams, and developer environments with typed Python.
---

A `.scry` file is Python syntax loaded through Scryr's manifest importer. It can import ordinary Python modules and sibling `.scry` modules, so normal composition and type checking still apply.

## Manifest: one component

A public `Manifest` variable becomes a block on the map. Its core fields are:

- `manifest_id`: stable organization-scoped identity for operational history.
- `name`: the display label and connection target.
- `icon`: an emoji or short icon string.
- `classification`: the block's architectural role.
- `tags`: useful categories for filtering and queries.
- `connections`: named links to other manifests.
- `forges`: developer-environment configurations used by the component.

Add detail through typed sections: `Info`, `Github`, `Metrics`, `CICD`, `Tests`, `Dependencies`, `Performance`, and `OtherDiagram`.

`Info` can describe version, language, frameworks, deployment, owning team, authentication, observability, infrastructure as code, scaling, docs, and external links.

## Diagram: one useful point of view

A public `Diagram` selects the blocks a reader should see together.

```python
from scryr import Diagram

system = Diagram(name="System", manifests=[web, api, database])
customer_path = Diagram(name="Customer path", manifests=[web, api])
```

For large systems, use multiple diagrams for system context, customer paths, operability, or ownership boundaries.

## Query-selected diagrams

Use `ManifestQuery` to select components by typed fields instead of maintaining a long list.

```python
from scryr import Diagram, ManifestQuery
from scryr.types import ProgrammingLanguage

python_services = Diagram(
    name="Python services",
    query=ManifestQuery(language=ProgrammingLanguage.python),
)
```

A diagram accepts either `manifests=[...]` or `query=...`, never both.

## Stable identity

Set `manifest_id` on every component that receives durable CI, test, or operational history:

```python
catalog = Manifest(manifest_id="services/catalog-api", name="Catalog API")
```

The ID remains stable if the display name changes. Use repository-relative, organization-scoped values with letters, digits, dots, underscores, colons, slashes, or hyphens.

## Split a large repository

The entrypoint can import local `.scry` and `.py` modules:

```python
from services.catalog import catalog_api
from services.checkout import checkout_api

commerce = Diagram(name="Commerce", manifests=[catalog_api, checkout_api])
```

Scryr's formatter, linter, and type checker follow reachable local imports. Public imported Scryr objects can also be exports, so unused-import warnings are suppressed for this workflow.

## Forge: generate project tooling

A public `Forge` models tools, environment variables, variables, tasks, settings, and plugins once.

```python
from scryr import Forge

local = Forge(
    name="Developer environment",
    tools={"python": "3.14", "node": "24", "postgres": "17"},
    tasks={
        "install": "uv sync && npm install",
        "dev": "npm run dev",
        "test": "pytest && npm test",
    },
)
```

Export the same declaration into useful project files:

```sh
scryr export mise --forge "Developer environment"
scryr export compose --forge "Developer environment"
scryr export devcontainer --forge "Developer environment"
```

## Python project mode

The CLI normally uses the embedded SDK. To use a repository's locked Python environment, add an empty `scryr.toml` beside a `pyproject.toml` that depends on `scryr` and an up-to-date `uv.lock`.

```text title="scryr.toml"
# Presence opts this directory into project mode.
```

Scryr then runs `uv sync --no-dev --locked` for that project.
