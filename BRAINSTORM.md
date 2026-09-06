## Goal Manifest File

the goal file with manifest, forge, flows, envvars, is below as a guide to other features:

This is stored as the scry file or index.scry
```python
"""Scryr sample for a three-tier app.

This file is intentionally written as ordinary Python so the Scryr CLI can parse,
type-check, lint, and execute it with the same toolchain used by project code.

The sample shows the core authoring model:

- Manifest describes an architectural unit.
- Deployment defines each stage's target and envvars.
- Task is the unit of executable work and always declares tools.
- Flow requires tasks and only adds triggers.
- Forge is the local and CI execution environment.
- CICD, Test, and Metric are signals attached directly to manifests.
- Diagram queries use typed field refs and enum values.
"""

from scryr import Manifest
from scryr.change import ChangePolicy, GitTracking, ReviewRule
from scryr.diagram import Diagram, ManifestQuery, relations, where
from scryr.environment import Deployment, PromotionPath
from scryr.flow import Flow, Trigger
from scryr.forge import Forge, ManifestTools, Node, Postgres, Python, Task, Uv
from scryr.manifest import Link
from scryr.saas import Classification
from scryr.signal import CICD, Metric, Test, Threshold
from scryr.types import (
    AuthType,
    CICDToolType,
    DeploymentTarget,
    IaCToolType,
    InterfaceType,
    LogAggregationType,
    MonitoringType,
    ProgrammingLanguage,
    SemVer,
    TracingType,
    Url,
    WebFramework,
)

SOURCE = Url("https://github.com/example/acme-three-tier")
DOCS = Url("https://docs.example.com/acme")

MANIFEST_WEB = "acme-web"
MANIFEST_API = "acme-api"
MANIFEST_DB = "acme-postgres"

WEB_TOOLS = [Node("22")]
API_TOOLS = [Python("3.13"), Uv("latest")]
DB_TOOLS = [Postgres("16")]
PROJECT_TOOLS = [*WEB_TOOLS, *API_TOOLS, *DB_TOOLS]

WEB_ENVVARS = {
    "WEB_BASE_URL": "http://localhost:5173",
    "API_BASE_URL": "http://localhost:8000",
}
API_ENVVARS = {
    "DJANGO_SETTINGS_MODULE": "acme.settings.local",
    "DATABASE_URL": "${DATABASE_URL}",
}
DB_ENVVARS = {
    "POSTGRES_DB": "acme",
    "POSTGRES_USER": "acme",
    "POSTGRES_PASSWORD": "acme",
}
PROJECT_ENVVARS = {**WEB_ENVVARS, **API_ENVVARS, **DB_ENVVARS}


WEB_DEPLOYMENTS = {
    "development": Deployment(
        target=DeploymentTarget.unknown,
        envvars=WEB_ENVVARS,
    ),
    "staging": Deployment(
        target=DeploymentTarget.cloudfront,
        envvars={
            **WEB_ENVVARS,
            "WEB_BASE_URL": "https://staging.example.com",
            "API_BASE_URL": "https://api.staging.example.com",
        },
    ),
    "production": Deployment(
        target=DeploymentTarget.cloudfront,
        envvars={
            **WEB_ENVVARS,
            "WEB_BASE_URL": "https://app.example.com",
            "API_BASE_URL": "https://api.example.com",
        },
    ),
}

API_DEPLOYMENTS = {
    "development": Deployment(
        target=DeploymentTarget.unknown,
        envvars={**API_ENVVARS, "DEBUG": "true"},
    ),
    "staging": Deployment(
        target=DeploymentTarget.ecs,
        envvars={
            **API_ENVVARS,
            "DEBUG": "false",
            "DJANGO_SETTINGS_MODULE": "acme.settings.staging",
        },
    ),
    "production": Deployment(
        target=DeploymentTarget.ecs,
        envvars={
            **API_ENVVARS,
            "DEBUG": "false",
            "DJANGO_SETTINGS_MODULE": "acme.settings.production",
        },
    ),
}

DB_DEPLOYMENTS = {
    "development": Deployment(
        target=DeploymentTarget.unknown,
        envvars=DB_ENVVARS,
    ),
    "staging": Deployment(
        target=DeploymentTarget.unknown,
        envvars=DB_ENVVARS,
    ),
    "production": Deployment(
        target=DeploymentTarget.unknown,
        envvars=DB_ENVVARS,
    ),
}


web_test_task = Task(
    name="web:test",
    manifests=[MANIFEST_WEB],
    tools=ManifestTools(),
    run=Node.npm("run", "test", "--", "--run"),
    working_dir="web",
    outputs=["web/coverage/lcov.info"],
)

web_build_task = Task(
    name="web:build",
    manifests=[MANIFEST_WEB],
    tools=ManifestTools(),
    run=Node.npm("run", "build"),
    working_dir="web",
    depends_on=[web_test_task],
    outputs=["web/dist"],
)

api_lint_task = Task(
    name="api:lint",
    manifests=[MANIFEST_API],
    tools=ManifestTools(),
    run=Uv.run("ruff", "check", "."),
    working_dir="api",
)

api_test_task = Task(
    name="api:test",
    manifests=[MANIFEST_API, MANIFEST_DB],
    tools=[*API_TOOLS, *DB_TOOLS],
    envvars={**API_ENVVARS, **DB_ENVVARS},
    services=[MANIFEST_DB],
    run=Uv.run("pytest", "tests", "--cov=acme"),
    working_dir="api",
    depends_on=[api_lint_task],
    outputs=["api/.coverage", "api/test-results.xml"],
)

api_migrate_task = Task(
    name="api:migrate",
    manifests=[MANIFEST_API, MANIFEST_DB],
    tools=[*API_TOOLS, *DB_TOOLS],
    envvars={**API_ENVVARS, **DB_ENVVARS},
    services=[MANIFEST_DB],
    run=Uv.run("python", "manage.py", "migrate", "--noinput"),
    working_dir="api",
)

integration_test_task = Task(
    name="integration:test",
    manifests=[MANIFEST_WEB, MANIFEST_API, MANIFEST_DB],
    tools=PROJECT_TOOLS,
    envvars=PROJECT_ENVVARS,
    services=[MANIFEST_DB],
    run=Uv.run("pytest", "tests/integration", "--base-url", "${WEB_BASE_URL}"),
    depends_on=[web_build_task, api_migrate_task, api_test_task],
    outputs=["test-results/integration.xml"],
)

release_check_task = Task(
    name="release:check",
    manifests=[MANIFEST_WEB, MANIFEST_API, MANIFEST_DB],
    tools=PROJECT_TOOLS,
    envvars=PROJECT_ENVVARS,
    services=[MANIFEST_DB],
    run=Uv.run("scryr", "verify", "--environment", "production"),
    depends_on=[web_build_task, api_test_task, integration_test_task],
)


main_flow = Flow(
    name="main",
    manifests=[MANIFEST_WEB, MANIFEST_API, MANIFEST_DB],
    tasks=[web_test_task, api_lint_task, api_test_task, web_build_task],
    triggers=[
        Trigger.pull_request(paths=["web/**", "api/**", "samples/three_tier/index.scry"]),
        Trigger.push(branches=["main"]),
    ],
)

integration_flow = Flow(
    name="integration",
    manifests=[MANIFEST_WEB, MANIFEST_API, MANIFEST_DB],
    tasks=[integration_test_task],
    triggers=[
        Trigger.pull_request(labels=["run-integration"]),
        Trigger.schedule(cron="0 7 * * 1-5"),
    ],
)

release_flow = Flow(
    name="release",
    manifests=[MANIFEST_WEB, MANIFEST_API, MANIFEST_DB],
    tasks=[release_check_task],
    triggers=[Trigger.tag(pattern="v*"), Trigger.manual()],
)


project_forge = Forge(
    name="acme-three-tier",
    tools=PROJECT_TOOLS,
    envvars=PROJECT_ENVVARS,
    services=[MANIFEST_DB],
    tasks=[
        web_test_task,
        web_build_task,
        api_lint_task,
        api_test_task,
        api_migrate_task,
        integration_test_task,
        release_check_task,
    ],
    flows=[main_flow, integration_flow, release_flow],
)

change_policy = ChangePolicy(
    source=GitTracking(
        repository=SOURCE,
        manifest_path="samples/three_tier/index.scry",
        default_branch="main",
        release_refs=["refs/tags/v*"],
    ),
    promotion=PromotionPath(
        environments=[
            "development",
            "staging",
            "production",
        ],
        required_flows=[main_flow, integration_flow, release_flow],
    ),
    review_rules=[
        ReviewRule(
            fields=[where.deployments, where.auth],
            environments=["production"],
            required_approvals=2,
        ),
        ReviewRule(
            manifests=[MANIFEST_DB],
            fields=[where.storage, where.backup_policy],
            required_approvals=1,
        ),
    ],
)


web = Manifest(
    id=MANIFEST_WEB,
    name="Acme Web",
    description="React customer frontend.",
    classification=Classification.public_ui,
    language=ProgrammingLanguage.typescript,
    framework=WebFramework.react,
    version=SemVer("2.8.0"),
    source=SOURCE / "tree/main/web",
    docs=DOCS / "web",
    owner_team="product-platform",
    deployments=WEB_DEPLOYMENTS,
    iac=IaCToolType.terraform,
    interface=InterfaceType.http,
    auth=AuthType.oidc,
    observability=[MonitoringType.datadog, TracingType.opentelemetry],
    logging=LogAggregationType.cloudwatch,
    min_replicas=2,
    max_replicas=12,
    connects_to=[MANIFEST_API],
    links=[
        Link(label="Application", url=Url("https://app.example.com")),
        Link(label="Runbook", url=DOCS / "web/runbook"),
    ],
    tools=WEB_TOOLS,
    envvars=WEB_ENVVARS,
    forge=project_forge,
    tasks=[web_test_task, web_build_task, integration_test_task, release_check_task],
    flows=[main_flow, integration_flow, release_flow],
    change_policy=change_policy,
    cicd=CICD(
        tool=CICDToolType.scryr,
        flow=main_flow,
        required_for=["staging", "production"],
    ),
    test=Test(
        task=web_test_task,
        coverage=Threshold.greater_than(85),
        required_for=["staging", "production"],
    ),
    metric=[
        Metric.name("p95_page_load_ms").less_than(1800),
        Metric.name("frontend_error_rate").less_than(0.01),
    ],
)

api = Manifest(
    id=MANIFEST_API,
    name="Acme API",
    description="Django JSON API for product, checkout, and account workflows.",
    classification=Classification.public_api,
    language=ProgrammingLanguage.python,
    framework=WebFramework.django,
    version=SemVer("2.8.0"),
    source=SOURCE / "tree/main/api",
    docs=DOCS / "api",
    owner_team="product-platform",
    deployments=API_DEPLOYMENTS,
    iac=IaCToolType.terraform,
    interface=InterfaceType.http,
    auth=AuthType.oidc,
    observability=[MonitoringType.datadog, TracingType.opentelemetry],
    logging=LogAggregationType.cloudwatch,
    min_replicas=3,
    max_replicas=20,
    connects_to=[MANIFEST_DB],
    links=[Link(label="OpenAPI", url=DOCS / "api/openapi")],
    tools=API_TOOLS,
    envvars=API_ENVVARS,
    forge=project_forge,
    tasks=[
        api_lint_task,
        api_test_task,
        api_migrate_task,
        integration_test_task,
        release_check_task,
    ],
    flows=[main_flow, integration_flow, release_flow],
    change_policy=change_policy,
    cicd=CICD(
        tool=CICDToolType.scryr,
        flow=main_flow,
        required_for=["staging", "production"],
    ),
    test=Test(
        task=api_test_task,
        coverage=Threshold.greater_than(90),
        required_for=["staging", "production"],
    ),
    metric=[
        Metric.name("p95_latency_ms").less_than(250),
        Metric.name("http_5xx_rate").less_than(0.005),
    ],
)

db = Manifest(
    id=MANIFEST_DB,
    name="Acme Postgres",
    description="Primary relational database for the Acme application.",
    classification=Classification.database,
    language=ProgrammingLanguage.sql,
    version=SemVer("16.4.0"),
    source=SOURCE / "tree/main/database",
    docs=DOCS / "database",
    owner_team="data-platform",
    deployments=DB_DEPLOYMENTS,
    iac=IaCToolType.terraform,
    interface=InterfaceType.postgres,
    auth=AuthType.password,
    observability=[MonitoringType.datadog],
    logging=LogAggregationType.cloudwatch,
    min_replicas=1,
    max_replicas=2,
    storage="500Gi encrypted gp3",
    backup_policy="point-in-time recovery, 35 day retention",
    links=[Link(label="Runbook", url=DOCS / "database/runbook")],
    tools=DB_TOOLS,
    envvars=DB_ENVVARS,
    forge=project_forge,
    tasks=[api_migrate_task, api_test_task, integration_test_task, release_check_task],
    flows=[main_flow, integration_flow, release_flow],
    change_policy=change_policy,
    cicd=CICD(
        tool=CICDToolType.scryr,
        flow=release_flow,
        required_for=["production"],
    ),
    test=Test(
        task=integration_test_task,
        assertion="migrations apply cleanly before integration tests",
        required_for=["staging", "production"],
    ),
    metric=[
        Metric.name("cpu_utilization").less_than(0.70),
        Metric.name("free_storage_percent").greater_than(0.20),
    ],
)


system_diagram = Diagram(
    name="System",
    query=ManifestQuery()
    .select(where.id, where.name, where.classification, where.owner_team)
    .from_manifests([web, api, db])
    .include(relations.connects_to)
    .order_by(where.classification, where.name),
)

production_readiness_diagram = Diagram(
    name="Production readiness",
    query=ManifestQuery()
    .from_manifests([web, api, db])
    .where(where.deployments.includes("production"))
    .include(relations.flows, relations.tasks, relations.metrics)
    .group_by(where.owner_team),
)

data_path_diagram = Diagram(
    name="Data path",
    query=ManifestQuery()
    .from_manifests([web, api, db])
    .where(
        where.classification.in_(
            [
                Classification.public_ui,
                Classification.public_api,
                Classification.database,
            ]
        )
    )
    .include(relations.connects_to)
    .layout("left-to-right"),
)


manifests = [web, api, db]
forges = [project_forge]
flows = [main_flow, integration_flow, release_flow]
change_policies = [change_policy]
diagrams = [system_diagram, production_readiness_diagram, data_path_diagram]

```
