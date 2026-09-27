# Scryr Manifest

The Python SDK defines architecture and laptop evidence in one typed `index.scry`.
The Rust `scryr serve` process executes declared collectors; constructing or
serializing SDK objects never runs tools or connects to a service.

```python
from scryr import Diagram, Info, Manifest, ProgrammingLanguage
from scryr.collectors import GitStatusCollector, PytestCollector, RuffCheckCollector

api = Manifest(
    manifest_id="services/api",
    name="Public API",
    info=Info(language=ProgrammingLanguage.python),
    repository=[GitStatusCollector()],
    checks=[RuffCheckCollector()],
    tests=[PytestCollector()],
)
architecture = Diagram(name="Local architecture", manifests=[api])
```

The six evidence fields are `repository`, `checks`, `metrics`, `tests`,
`dependencies`, and `performance`. Each accepts a typed list of concrete
integration classes from `scryr.collectors`; empty lists hide their cards.
`Info` and `OtherDiagram` describe architecture. `Forge` describes tools/tasks.

Read the [SDK guide](scryr/README.md) for collectors, schedules, inventory
references, and their wire format. The [local development sample](tests/samples/local_development/index.scry)
shows all six cards. The [GitHub Actions sample](tests/samples/github_actions/index.scry)
keeps remote workflow context in Repository and local lint in Checks.

## Development

Run mise commands from the repository root:

```sh
mise run contribute:setup
mise run verify:manifest
```

A `.scry` file uses Python syntax and may import normal Python or sibling `.scry`
modules. It is executable Python, so only load trusted source. Public `Manifest`,
`Diagram`, and `Forge` values are serialized; public collectors live inside the
manifest's section lists, not as new top-level constructs.

The Rust CLI manages pinned uv/Python and a per-project virtual environment under
`<scryr-dir>/python-envs/<project-id>/.venv`. These support manifest evaluation;
collectors use the project's actual tool environment. First execution can require
network access to provision the SDK runtime. See the
[CLI reference](../crystal/crystal-cli/README.md).

This workspace's `scryr.toml` opts SDK evaluation into the locked Python workspace.
It is contributor/runtime selection, not a second authored collector configuration.
Normal standalone usage needs only the SDK declarations in `index.scry`.
