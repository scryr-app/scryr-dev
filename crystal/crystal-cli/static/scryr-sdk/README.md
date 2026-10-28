# Scryr

Scryr is a Python package for defining architecture manifests as typed Pydantic
models. Applications can use it as a library.

Use `Classification` from `scryr.types` for architectural roles and interfaces:

```python
from scryr import Manifest
from scryr.types import Classification

api = Manifest(name="API", classification=Classification.rest)
edge = Manifest(name="Edge", classification=Classification.reverse_proxy)
```

`Classification` includes existing roles such as `public_api`, `database`, and
`worker`, plus interface choices such as `rest`, `graphql`, `grpc`, and `sql`.
Additional infrastructure roles include:

- Traffic and networking: `load_balancer`, `reverse_proxy`, `cdn`, `dns`.
- Security and identity: `firewall`, `identity_provider`, `secrets_manager`.
- Background processing: `data_pipeline`, `workflow_engine`.
- Observability: `monitoring`, `log_collector`.
- Machine learning: `model_service`.

Values serialize as strings. `InterfaceType` remains an import alias for
`Classification`; its former tuple values and interface metadata properties
(`category`, `id`, and `description`) are no longer supported.

## Info fields

Enum fields are typed lists and default to empty. `deployments` combines hosting
choices and infrastructure-as-code tools; `metric_tools` combines monitoring,
tracing, and log aggregation without duplicate tools. `owners` is a single string.

```python
from scryr import Info
from scryr.types import DeploymentTarget, Markdown, MetricTool, ProgrammingLanguage, WebFramework

info = Info(
    description=Markdown("**Customer API** and background processing."),
    languages=[ProgrammingLanguage.python, ProgrammingLanguage.sql],
    frameworks=[WebFramework.fastapi],
    deployments=[DeploymentTarget.ec2, DeploymentTarget.terraform],
    metric_tools=[MetricTool.prometheus, MetricTool.jaeger, MetricTool.loki],
    owners="Platform engineering",
)
```

Migrate `language` to `languages=[...]`, and combine `deployment` and `iac_tool`
into `deployments=[...]`. Combine `monitoring`, `tracing`, and `log_aggregation`
into `metric_tools=[...]`; omit old `none` values and use `MetricTool.sumo_logic`
for Sumo Logic. Rename `owner_team` to `owners` and remove `auth_type`.
The old constructor fields are rejected. Update stored editable Python sources
before rerunning them; the server can still display older stored JSON diagrams.
JSON uses `metricTools`; the other new names match the Python names.
