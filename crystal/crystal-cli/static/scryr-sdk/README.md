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

## GitHub Builds & releases

Add `delivery=GitHubDelivery(...)` and a stable `manifest_id` to a Manifest to
enable the Builds & releases card. Import the typed configuration models from
`scryr.delivery`. See `../tests/samples/github_delivery/index.scry` for a complete
example, including typed delivery tasks and a pinned GitHub CLI tool. Replace its example
repository and workflow selectors before collecting.

The definition selects the repository, build workflow, artifact names, release
filters, deployment environments, evidence workflows and rollback policy.
`GitHubDelivery.tasks` owns `collect` (`DeliveryTask`) and `verify`
(`DeliveryVerificationTask`) definitions through `DeliveryTasks`. Their typed
fields control collection limits and attestation verification; there are no shell
commands or duplicated manifest selectors in the task definitions. Forge only
installs the tools. `DeliveryReportSchema` names the supported evidence formats.
Constructing or editing it never contacts GitHub. Authenticate `gh` outside the
manifest, then collect with:

```sh
scryr report delivery --path index.scry --manifest payments-api --task collect --dry-run
scryr report delivery --path index.scry --manifest payments-api --task collect
```

The first command previews the normalized observation; the second persists it
to the configured Scryr endpoint. GitHub provides build runs, jobs, artifact
inventory, releases and deployment records. The card keeps reported deployment
success separate from observed runtime state. Missing runtime and compatibility
evidence remains unknown. GitHub CLI is open source; the data service is GitHub.

Use `--task verify` to run the configured verification task and download
release assets and verify their content digests and build-workflow attestations.
Without verification, collected native artifacts have unknown provenance.
Omitting `--task` selects `collect`. Existing definitions without `tasks` default
to a limit of 20, with verification disabled for `collect` and enabled for `verify`.
`--limit` and `--verify-attestations` remain explicit command-line overrides.

For explicit source-to-artifact-to-deployment relationships, create a
`DeliveryLink` and include its JSON under `payload.scryr` when creating the GitHub
deployment. Omit `deployment_id` using `model_dump(mode="json", exclude_none=True)`;
GitHub assigns the ID and the collector binds the payload to its enclosing
deployment. Supply the actual build run/attempt, source revision, deployed
content digest and configuration revision. A run's head commit or an Actions
archive digest must not be substituted for the contained artifact's identity.

For runtime and compatibility evidence, workflows can construct
`DeliveryRuntimeEvidence` and `DeliveryCompatibilityEvidence` from actual probe
and test results, write `model_dump_json()` to `report.json`, and upload that file
as the configured Actions artifact. Report schemas are `scryr.runtime.v1` and
`scryr.compatibility.v1`; timestamps require time zones. Runtime reports carry
the observed digest distribution and schema/configuration identities.
Compatibility reports bind a candidate digest to the exact schema/configuration
tested and a versioned test suite. Set `candidate_configuration_revision` when
the rollback would restore a different configuration; omitting it only qualifies
when the candidate and current configuration are identical. These are observations, not fields to hardcode
in `index.scry`.

Rollback eligibility is derived from configured requirements and matching
evidence; it is not a guarantee that rollback is safe. Provenance remains unknown
unless verified, and reproducibility is not inferred from successful builds.
Collection does not deploy, execute rollback, or run commands found in reports.
