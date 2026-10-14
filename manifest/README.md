# Scryr Manifest

Define your architecture in a simple manifest format.

The installable Python package lives in `scryr/` and exposes the public API from
`scryr`:

```python
from scryr import Diagram, Manifest

database = Manifest(name="Database")
api = Manifest(name="Public API", connections=[database], tags=["backend"])
system = Diagram(name="System", manifests=[api, database])
```

Manifests describe component identity, classification, tags, connections, and Forge references.

For local development:

```bash
mise run contribute:setup
mise run verify:test:manifest
cd manifest
uv run python -m scryr.cli tests/samples/open_saas/index.scry --json
```

Manifest files may also use the `.scry` extension. A `.scry` file is Python
syntax loaded through Scryr's manifest importer, so it can import normal Python
modules and sibling `.scry` modules:

```python
from scryr import Manifest
from shared_blocks import database
from labels import service_name

api = Manifest(name=f"{service_name} API")
```

When the Scryr CLI executes manifests, the Rust CLI manages its own Python
runtime. It installs Scryr's pinned `uv` into `<scryr-dir>/bin`, installs
uv-managed Python into `<scryr-dir>/python`, and runs each manifest project in
an isolated virtual environment under
`<scryr-dir>/python-envs/<project-id>/.venv`. The project id is derived from the
manifest project directory, which keeps dependencies for different manifest
projects separate while still reusing the same environment for repeated runs of
one project.

The Rust `crystal-cli` exposes higher-level generation commands for map uploads,
`mise.toml`, Docker Compose, devcontainers, schemas, and type metadata. See
[`../crystal/crystal-cli/README.md`](../crystal/crystal-cli/README.md) for the full
CLI reference.

The workspace includes `scryr.toml` to opt local Rust CLI generation into this
Python workspace and its locked dependencies. Other directories use the CLI's
embedded SDK unless they also contain `scryr.toml`; `pyproject.toml` alone does
not enable project mode.
