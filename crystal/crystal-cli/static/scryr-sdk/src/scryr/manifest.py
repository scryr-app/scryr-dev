"""Manifest data models."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from enum import Enum
from typing import TYPE_CHECKING, Any, ClassVar, Literal, Never, override

from pydantic import (
    AliasChoices,
    BaseModel,
    ConfigDict,
    Field,
    field_serializer,
    field_validator,
    model_validator,
)

from .github import GithubActionsLog  # noqa: TC001 - Pydantic resolves this at runtime.
from .metrics_source import PostHogSource, PrometheusSource  # noqa: TC001
from .types import (
    AuthType,
    CalendarVersion,
    CICDToolType,
    Classification,
    DeploymentTarget,
    IaCToolType,
    IncrementalVersion,
    Label,
    LogAggregationType,
    Markdown,
    MonitoringType,
    ProgrammingLanguage,
    SemVer,
    TracingType,
    Url,
    Version,
    WebFramework,
)

if TYPE_CHECKING:
    import builtins
    from collections.abc import Callable

    from pydantic.main import IncEx

type ForgePrimitive = str | int | float | bool
type ForgeEnvValue = ForgePrimitive | ForgeEnvDirective
type ForgeToolSpec = str | Sequence[str] | ForgeTool
type ForgeTaskRunStep = str | ForgeTaskCommand | ForgeTaskBatch
type ForgeTaskRun = str | Sequence[ForgeTaskRunStep]
type ForgeTaskDependency = str | ForgeTaskCommand
type ForgeTaskDependencyList = ForgeTaskDependency | Sequence[ForgeTaskDependency]
type ForgeTaskSpec = str | Sequence[ForgeTaskRunStep] | ForgeTask

type BuildStatus = Literal["passing", "failing", "pending"]
type DeployStatus = Literal["deployed", "deploying", "failed"]
type CoverageTrend = Literal["up", "down", "stable"]
type DependencySeverity = Literal["critical", "high", "medium", "low", "none"]
type LicenseCompliance = Literal["compliant", "warning", "violation"]

_QUERY_MISSING = object()


def _parse_version_value(value: object) -> Version:
    """Coerce incoming wire values into strongly typed version models."""
    if isinstance(value, str):
        for constructor in (SemVer, CalendarVersion, IncrementalVersion):
            try:
                return constructor(value)
            except ValueError:
                continue

        msg = (
            "Unsupported version format. Supported: SemVer (MAJOR.MINOR.PATCH), "
            "Calendar (YEAR.MONTH), Incremental (Numbering). "
            f"Input: {value!r}."
        )
        raise ValueError(msg)
    if isinstance(value, (SemVer, CalendarVersion, IncrementalVersion)):
        return value

    msg = "version must be a version string or a supported version model"
    raise TypeError(msg)


class _Section(BaseModel):
    """Base model for optional Manifest section data."""

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )


class Link(BaseModel):
    """A data model representing a site link with name and URL."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    site_name: Label | str = Field(default=Label(""), description="Display name for the site")
    http_url: Url | str = Field(default=Url(""), description="URL of the site")


class Info(_Section):
    """Manifest section for Info data."""

    description: Markdown | str = Field(default=Markdown(""), description="Short description")
    version: Version = Field(
        default_factory=lambda: SemVer.parse("0.1.0"),
        description=(
            "Service version supporting SemVer (MAJOR.MINOR.PATCH), "
            "Calendar (YEAR.MONTH), or Incremental (Numbering)"
        ),
    )
    language: ProgrammingLanguage = Field(
        default=ProgrammingLanguage.python, description="Primary programming language"
    )
    frameworks: list[WebFramework] = Field(
        default_factory=list, description="Web frameworks in use"
    )
    deployment: DeploymentTarget = Field(
        default=DeploymentTarget.ec2, description="Deployment target"
    )
    owner_team: Label | str = Field(
        default=Label(""),
        serialization_alias="ownerTeam",
        validation_alias=AliasChoices("owner_team", "ownerTeam"),
        description="Team responsible for this service",
    )
    auth_type: AuthType = Field(
        default=AuthType.api_key,
        serialization_alias="authType",
        validation_alias=AliasChoices("auth_type", "authType"),
        description="Authentication mechanism",
    )
    monitoring: MonitoringType = Field(
        default=MonitoringType.none, description="Monitoring platform"
    )
    tracing: TracingType = Field(default=TracingType.none, description="Distributed tracing tool")
    log_aggregation: LogAggregationType = Field(
        default=LogAggregationType.none,
        serialization_alias="logAggregation",
        validation_alias=AliasChoices("log_aggregation", "logAggregation"),
        description="Log aggregation tool",
    )
    iac_tool: IaCToolType = Field(
        default=IaCToolType.none,
        serialization_alias="iacTool",
        validation_alias=AliasChoices("iac_tool", "iacTool"),
        description="Infrastructure as Code tooling",
    )
    max_replicas: int = Field(
        default=1,
        ge=1,
        serialization_alias="maxReplicas",
        validation_alias=AliasChoices("max_replicas", "maxReplicas"),
        description="Maximum number of service replicas",
    )
    min_replicas: int = Field(
        default=1,
        ge=1,
        serialization_alias="minReplicas",
        validation_alias=AliasChoices("min_replicas", "minReplicas"),
        description="Minimum number of service replicas",
    )
    docs: list[Url | str] = Field(default_factory=list, description="Documentation URLs")
    links: list[Link] = Field(default_factory=list, description="External links")

    @field_validator("version", mode="before")
    @classmethod
    def _parse_version_input(cls, value: object) -> Version:
        return _parse_version_value(value)

    @field_serializer("version")
    def _serialize_version(self, value: Version) -> str:
        return str(value)

    @field_validator("deployment", mode="before")
    @classmethod
    def _parse_deployment(cls, value: object) -> object:
        if isinstance(value, str):
            return next((target for target in DeploymentTarget if target.id == value), value)
        return value

    @field_validator("frameworks", mode="before")
    @classmethod
    def _parse_frameworks(cls, value: object) -> object:
        if isinstance(value, list):
            return [
                next((framework for framework in WebFramework if framework.framework == item), item)
                if isinstance(item, str)
                else item
                for item in value
            ]
        return value

    @field_serializer("frameworks")
    def _serialize_frameworks(self, value: list[WebFramework]) -> list[str]:
        return [framework.framework for framework in value]

    @field_serializer("deployment")
    def _serialize_deployment(self, value: DeploymentTarget) -> str:
        return value.id

    @field_serializer(
        "language",
        "auth_type",
        "monitoring",
        "tracing",
        "log_aggregation",
        "iac_tool",
    )
    def _serialize_value_enum(self, value: object) -> object:
        return getattr(value, "value", value)

    @model_validator(mode="after")
    def _validate_replica_bounds(self) -> Info:
        """Ensure replica bounds remain internally consistent."""
        if self.min_replicas > self.max_replicas:
            msg = "min_replicas must be less than or equal to max_replicas"
            raise ValueError(msg)
        return self


class Github(_Section):
    """Manifest section for Github data."""

    enabled: bool | None = Field(default=None, description="Whether Github data is enabled")
    repo_url: Url | str | None = Field(
        default=None,
        serialization_alias="repoUrl",
        validation_alias=AliasChoices("repo_url", "repoUrl"),
        description="Repository URL",
    )
    stars: int | None = Field(default=None, ge=0, description="Number of stars")
    forks: int | None = Field(default=None, ge=0, description="Number of forks")
    open_issues: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="openIssues",
        validation_alias=AliasChoices("open_issues", "openIssues"),
        description="Number of open issues",
    )
    open_prs: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="openPRs",
        validation_alias=AliasChoices("open_prs", "openPRs"),
        description="Number of open PRs",
    )
    last_commit: str | None = Field(
        default=None,
        serialization_alias="lastCommit",
        validation_alias=AliasChoices("last_commit", "lastCommit"),
        description="Last commit date",
    )
    primary_language: ProgrammingLanguage | str | None = Field(
        default=None,
        serialization_alias="primaryLanguage",
        validation_alias=AliasChoices("primary_language", "primaryLanguage"),
        description="Primary language",
    )
    lines_of_code: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="linesOfCode",
        validation_alias=AliasChoices("lines_of_code", "linesOfCode"),
        description="Lines of code",
    )
    coverage: float | None = Field(default=None, ge=0, le=100, description="Test coverage")
    vulnerabilities: int | None = Field(
        default=None,
        ge=0,
        description="Security vulnerabilities count",
    )
    outdated_deps: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="outdatedDeps",
        validation_alias=AliasChoices("outdated_deps", "outdatedDeps"),
        description="Outdated dependencies count",
    )
    active_contributors: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="activeContributors",
        validation_alias=AliasChoices("active_contributors", "activeContributors"),
        description="Active contributors",
    )
    latest_release: Version | str | None = Field(
        default=None,
        serialization_alias="latestRelease",
        validation_alias=AliasChoices("latest_release", "latestRelease"),
        description="Latest release version",
    )
    license: str | None = Field(default=None, description="License type")
    build_status: BuildStatus | None = Field(
        default=None,
        serialization_alias="buildStatus",
        validation_alias=AliasChoices("build_status", "buildStatus"),
        description="Build status",
    )

    @field_serializer("primary_language")
    def _serialize_primary_language(self, value: ProgrammingLanguage | str | None) -> str | None:
        if value is None:
            return None
        return value.value if isinstance(value, ProgrammingLanguage) else str(value)

    @field_serializer("latest_release")
    def _serialize_latest_release(self, value: Version | str | None) -> str | None:
        return None if value is None else str(value)


class Metrics(_Section):
    """Manifest section for Metrics data."""

    provider: PrometheusSource | None = None

    enabled: bool | None = Field(default=None, description="Whether Metrics data is enabled")
    response_time_p50: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="responseTimeP50",
        validation_alias=AliasChoices("response_time_p50", "responseTimeP50", "p50"),
        description="Response time p50 in ms",
    )
    response_time_p95: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="responseTimeP95",
        validation_alias=AliasChoices("response_time_p95", "responseTimeP95", "p95"),
        description="Response time p95 in ms",
    )
    response_time_p99: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="responseTimeP99",
        validation_alias=AliasChoices("response_time_p99", "responseTimeP99", "p99"),
        description="Response time p99 in ms",
    )
    request_rate: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="requestRate",
        validation_alias=AliasChoices("request_rate", "requestRate"),
        description="Request rate",
    )
    error_rate: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="errorRate",
        validation_alias=AliasChoices("error_rate", "errorRate"),
        description="Error rate percentage",
    )
    success_rate: float | None = Field(
        default=None,
        ge=0,
        le=100,
        serialization_alias="successRate",
        validation_alias=AliasChoices("success_rate", "successRate"),
        description="Success rate percentage",
    )
    uptime: float | None = Field(default=None, ge=0, le=100, description="Uptime percentage")
    active_connections: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="activeConnections",
        validation_alias=AliasChoices("active_connections", "activeConnections"),
        description="Active connections count",
    )
    cpu_usage: float | None = Field(
        default=None,
        ge=0,
        le=100,
        serialization_alias="cpuUsage",
        validation_alias=AliasChoices("cpu_usage", "cpuUsage"),
        description="CPU usage percentage",
    )
    memory_usage: float | None = Field(
        default=None,
        ge=0,
        le=100,
        serialization_alias="memoryUsage",
        validation_alias=AliasChoices("memory_usage", "memoryUsage"),
        description="Memory usage percentage",
    )


class CICD(_Section):
    """Manifest section for CICD data."""

    reports: dict[str, Any] | None = Field(
        default=None, description="Current durable reports by kind and scope"
    )

    github_actions: GithubActionsLog | None = Field(
        default=None,
        serialization_alias="githubActions",
        validation_alias=AliasChoices("github_actions", "githubActions"),
        description="Workflow runs and their observed status history",
    )

    platform: CICDToolType | str | None = Field(default=None, description="CI/CD platform name")
    build_status: BuildStatus | None = Field(
        default=None,
        serialization_alias="buildStatus",
        validation_alias=AliasChoices("build_status", "buildStatus"),
        description="Build status",
    )
    last_build: str | None = Field(
        default=None,
        serialization_alias="lastBuild",
        validation_alias=AliasChoices("last_build", "lastBuild"),
        description="Last build timestamp",
    )
    deploy_status_prod: DeployStatus | None = Field(
        default=None,
        serialization_alias="deployStatusProd",
        validation_alias=AliasChoices("deploy_status_prod", "deployStatusProd"),
        description="Production deployment status",
    )
    deploy_status_staging: DeployStatus | None = Field(
        default=None,
        serialization_alias="deployStatusStaging",
        validation_alias=AliasChoices("deploy_status_staging", "deployStatusStaging"),
        description="Staging deployment status",
    )
    deploy_frequency: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="deployFrequency",
        validation_alias=AliasChoices("deploy_frequency", "deployFrequency"),
        description="Deployments per week",
    )
    pipeline_duration: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="pipelineDuration",
        validation_alias=AliasChoices("pipeline_duration", "pipelineDuration"),
        description="Pipeline duration in minutes",
    )
    failed_builds: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="failedBuilds",
        validation_alias=AliasChoices("failed_builds", "failedBuilds"),
        description="Failed builds count",
    )

    @field_serializer("platform")
    def _serialize_platform(self, value: CICDToolType | str | None) -> str | None:
        if value is None:
            return None
        return value.value if isinstance(value, CICDToolType) else str(value)


class Tests(_Section):
    """Manifest section for Tests data."""

    reports: dict[str, Any] | None = Field(
        default=None, description="Current durable reports by kind and scope"
    )

    errors: int | None = Field(default=None, ge=0, description="Errored tests")
    skipped: int | None = Field(default=None, ge=0, description="Skipped tests")
    total: int | None = Field(default=None, ge=0, description="Total number of tests")
    passing: int | None = Field(default=None, ge=0, description="Number of passing tests")
    failing: int | None = Field(default=None, ge=0, description="Number of failing tests")
    coverage: float | None = Field(default=None, ge=0, le=100, description="Test coverage")
    coverage_trend: CoverageTrend | None = Field(
        default=None,
        serialization_alias="coverageTrend",
        validation_alias=AliasChoices("coverage_trend", "coverageTrend"),
        description="Coverage trend indicator",
    )
    flaky_tests: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="flakyTests",
        validation_alias=AliasChoices("flaky_tests", "flakyTests"),
        description="Number of flaky tests",
    )
    execution_time: float | None = Field(
        default=None,
        ge=0,
        serialization_alias="executionTime",
        validation_alias=AliasChoices("execution_time", "executionTime"),
        description="Test execution time in seconds",
    )
    last_run: str | None = Field(
        default=None,
        serialization_alias="lastRun",
        validation_alias=AliasChoices("last_run", "lastRun"),
        description="Last test run time",
    )


class Dependencies(_Section):
    """Manifest section for Dependencies data."""

    reports: dict[str, Any] | None = Field(
        default=None, description="Current durable reports by kind and scope"
    )

    open_alerts: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="openAlerts",
        validation_alias=AliasChoices("open_alerts", "openAlerts"),
    )
    total_deps: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="totalDeps",
        validation_alias=AliasChoices("total_deps", "totalDeps"),
        description="Total number of dependencies",
    )
    outdated_deps: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="outdatedDeps",
        validation_alias=AliasChoices("outdated_deps", "outdatedDeps"),
        description="Number of outdated dependencies",
    )
    vulnerable_deps: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="vulnerableDeps",
        validation_alias=AliasChoices("vulnerable_deps", "vulnerableDeps"),
        description="Number of dependencies with vulnerabilities",
    )
    max_severity: DependencySeverity | None = Field(
        default=None,
        serialization_alias="maxSeverity",
        validation_alias=AliasChoices("max_severity", "maxSeverity"),
        description="Highest severity vulnerability level",
    )
    direct_deps: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="directDeps",
        validation_alias=AliasChoices("direct_deps", "directDeps"),
        description="Number of direct dependencies",
    )
    transitive_deps: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="transitiveDeps",
        validation_alias=AliasChoices("transitive_deps", "transitiveDeps"),
        description="Number of transitive dependencies",
    )
    update_lag: int | None = Field(
        default=None,
        ge=0,
        serialization_alias="updateLag",
        validation_alias=AliasChoices("update_lag", "updateLag"),
        description="Dependency update lag in days",
    )
    license_compliance: LicenseCompliance | None = Field(
        default=None,
        serialization_alias="licenseCompliance",
        validation_alias=AliasChoices("license_compliance", "licenseCompliance"),
        description="License compliance status",
    )


class Performance(_Section):
    """Manifest section for Performance data."""

    cpu_history: list[float] | None = Field(
        default=None,
        serialization_alias="cpuHistory",
        validation_alias=AliasChoices("cpu_history", "cpuHistory"),
        description="CPU usage history",
    )
    cpu_current: float | None = Field(
        default=None,
        ge=0,
        le=100,
        serialization_alias="cpuCurrent",
        validation_alias=AliasChoices("cpu_current", "cpuCurrent"),
        description="Current CPU usage percentage",
    )
    cpu_avg: float | None = Field(
        default=None,
        ge=0,
        le=100,
        serialization_alias="cpuAvg",
        validation_alias=AliasChoices("cpu_avg", "cpuAvg"),
        description="Average CPU usage percentage",
    )
    cpu_peak: float | None = Field(
        default=None,
        ge=0,
        le=100,
        serialization_alias="cpuPeak",
        validation_alias=AliasChoices("cpu_peak", "cpuPeak"),
        description="Peak CPU usage percentage",
    )
    memory_usage: float | None = Field(
        default=None,
        ge=0,
        le=100,
        serialization_alias="memoryUsage",
        validation_alias=AliasChoices("memory_usage", "memoryUsage"),
        description="Current memory usage percentage",
    )
    time_window: str | None = Field(
        default=None,
        serialization_alias="timeWindow",
        validation_alias=AliasChoices("time_window", "timeWindow"),
        description="Time window label",
    )


class OtherDiagram(_Section):
    """Manifest section for OtherDiagram data."""

    diagrams: list[Label | str] | None = Field(
        default=None,
        description="Names of the diagrams this block belongs to",
    )


class ForgeEnvDirective(BaseModel):
    """A typed mise `[env]` or `[vars]` directive value."""

    model_config = ConfigDict(extra="allow", validate_default=True)

    value: ForgePrimitive | list[str] | None = Field(
        default=None,
        description="Explicit value for the variable or var directive",
    )
    default: str | int | bool | None = Field(
        default=None,
        description="Fallback value used only when no earlier value is set",
    )
    required: str | bool | None = Field(
        default=None,
        description="Required-variable marker or error message",
    )
    path: str | list[str] | None = Field(
        default=None,
        description="Path value or path list for mise path-like directives",
    )
    tools: bool = Field(
        default=False,
        description="Resolve this directive after tool-provided environment values are available",
    )
    redact: bool = Field(default=False, description="Redact this value from mise task output")


class ForgeTool(BaseModel):
    """Detailed mise `[tools]` entry."""

    model_config = ConfigDict(extra="allow", validate_default=True)

    version: str | list[str] = Field(description="Tool version or versions")
    os: str | list[str] | None = Field(default=None, description="Operating systems for this tool")
    depends: list[str] = Field(
        default_factory=list,
        description="Tools that should be installed before this tool",
    )
    install_env: dict[str, ForgePrimitive] = Field(
        default_factory=dict,
        description="Environment variables used while installing this tool",
    )
    postinstall: str | list[str] | None = Field(
        default=None,
        description="Command or commands to run after installing this tool",
    )


class ForgeTaskCommand(BaseModel):
    """Structured mise task reference used in `run`, `depends`, and related fields."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    task: str = Field(description="Task name to run")
    args: list[str] = Field(default_factory=list, description="Arguments passed to the task")
    env: dict[str, ForgePrimitive] = Field(
        default_factory=dict,
        description="Environment variables passed to the task reference",
    )


class ForgeTaskBatch(BaseModel):
    """Parallel task batch used inside a mise task `run` list."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    tasks: list[str] = Field(default_factory=list, description="Task names to run in parallel")


class ForgeTaskConfirm(BaseModel):
    """Detailed mise task confirmation prompt."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    message: str = Field(description="Prompt shown before running the task")
    default: str | bool | None = Field(default=None, description="Default confirmation response")


class ForgeTask(BaseModel):
    """Detailed mise `[tasks.<name>]` entry."""

    model_config = ConfigDict(extra="allow", validate_default=True)

    run: ForgeTaskRun | None = Field(default=None, description="Command or commands to run")
    run_windows: ForgeTaskRun | None = Field(
        default=None,
        description="Windows-specific command or commands to run",
    )
    description: Markdown | str = Field(default=Markdown(""), description="Task description")
    alias: str | list[str] | None = Field(default=None, description="Task alias or aliases")
    depends: ForgeTaskDependencyList | None = Field(
        default=None,
        description="Tasks that must run before this task",
    )
    depends_post: ForgeTaskDependencyList | None = Field(
        default=None,
        description="Tasks that run after this task completes",
    )
    wait_for: ForgeTaskDependencyList | None = Field(
        default=None,
        description="Optional tasks to wait for if already running",
    )
    env: dict[str, ForgeEnvValue] = Field(
        default_factory=dict,
        description="Task-local environment variables",
    )
    tools: dict[str, ForgeToolSpec] = Field(
        default_factory=dict,
        description="Task-local tool versions",
    )
    dir: str | None = Field(default=None, description="Task working directory")
    hide: bool = Field(default=False, description="Hide this task from task listings")
    confirm: str | ForgeTaskConfirm | None = Field(
        default=None,
        description="Confirmation prompt shown before running the task",
    )
    file: str | None = Field(default=None, description="External script file to execute")
    raw: bool = Field(default=False, description="Connect task directly to stdio")
    raw_args: bool = Field(default=False, description="Pass task arguments through verbatim")
    interactive: bool = Field(default=False, description="Run with exclusive interactive stdio")
    sources: str | list[str] | None = Field(default=None, description="Task input files or globs")
    outputs: str | list[str] | None = Field(default=None, description="Task output files or globs")
    shell: str | None = Field(default=None, description="Shell used to run inline task commands")
    quiet: bool = Field(default=False, description="Suppress mise task runner output")
    silent: bool | Literal["stdout", "stderr"] = Field(
        default=False,
        description="Suppress task output",
    )
    output: str | None = Field(default=None, description="Task output style")
    usage: str | None = Field(default=None, description="Task argument usage specification")


class Forge(BaseModel):
    """A top-level Scryr construct wrapping a fully typed mise.toml file."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    name: Label | str = Field(default=Label(""), description="Display name for this forge")
    description: Markdown | str = Field(default=Markdown(""), description="Short description")
    file_name: str = Field(default="mise.toml", description="Target mise config filename")
    tools: dict[str, ForgeToolSpec] = Field(
        default_factory=dict,
        description="mise `[tools]` table",
    )
    env: dict[str, ForgeEnvValue] = Field(
        default_factory=dict,
        description="mise `[env]` table",
    )
    vars: dict[str, ForgeEnvValue] = Field(
        default_factory=dict,
        description="mise `[vars]` table",
    )
    tasks: dict[str, ForgeTaskSpec] = Field(
        default_factory=dict,
        description="mise `[tasks]` table",
    )
    settings: dict[str, object] = Field(
        default_factory=dict,
        description="mise `[settings]` table",
    )
    plugins: dict[str, str] = Field(
        default_factory=dict,
        description="mise `[plugins]` table",
    )
    redactions: list[str] = Field(
        default_factory=list,
        description="Environment variable patterns that should be redacted",
    )
    raw: dict[str, object] = Field(
        default_factory=dict,
        description="Additional mise TOML sections not yet modeled by Scryr",
    )

    def to_mise_dict(self) -> builtins.dict[str, Any]:
        """Return only the mise.toml content represented by this forge."""
        data = self.model_dump(mode="json", exclude={"name", "description", "file_name", "raw"})
        return {**data, **self.raw}

    def to_dict(self) -> builtins.dict[str, Any]:
        """Return a plain dict (JSON-safe) representation of this forge."""
        import json

        return json.loads(self.model_dump_json())


class Manifest(BaseModel):
    """A data model representing a single Scryr block."""

    Info: ClassVar[type[Info]] = Info
    Github: ClassVar[type[Github]] = Github
    Metrics: ClassVar[type[Metrics]] = Metrics
    CICD: ClassVar[type[CICD]] = CICD
    Tests: ClassVar[type[Tests]] = Tests
    Dependencies: ClassVar[type[Dependencies]] = Dependencies
    Performance: ClassVar[type[Performance]] = Performance
    OtherDiagram: ClassVar[type[OtherDiagram]] = OtherDiagram

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )

    manifest_id: str | None = Field(
        default=None,
        min_length=1,
        max_length=256,
        pattern=r"^[A-Za-z0-9][A-Za-z0-9._:/-]*$",
        serialization_alias="manifestId",
        validation_alias=AliasChoices("manifest_id", "manifestId"),
        description="Stable, organization-scoped identity for operational history",
    )

    # Core identity and graph topology.
    name: Label | str = Field(default=Label(""), description="Display name for the component")
    icon: Label | str = Field(default=Label(""), description="Emoji or small icon string")
    classification: Classification = Field(
        default=Classification.public_api,
        description="Block classification for architecture visualization",
    )
    tags: list[Label | str] = Field(
        default_factory=list, description="Arbitrary tags for categorization and filtering"
    )
    connections: list[Manifest] = Field(
        default_factory=list,
        description="Named connections to other components or services",
    )
    forges: list[Label | str] = Field(
        default_factory=list,
        description="Named forge configurations this component uses",
    )

    # Manifest sections. Names intentionally match the map surface, but remain
    # domain-level Manifest section models rather than UI component references.
    info: Info = Field(default_factory=Info)
    github: Github | None = Field(default=None)
    analytics: PostHogSource | None = Field(default=None)
    metrics: Metrics | None = Field(default=None)
    cicd: CICD | None = Field(default=None)
    tests: Tests | None = Field(default=None)
    dependencies: Dependencies | None = Field(default=None)
    performance: Performance | None = Field(default=None)
    other_diagram: OtherDiagram | None = Field(
        default=None,
        serialization_alias="otherDiagram",
        validation_alias=AliasChoices("other_diagram", "otherDiagram"),
    )

    def __init__(  # noqa: PLR0913
        self,
        *,
        manifest_id: str | None = None,
        name: Label | str = "",
        icon: Label | str = "",
        classification: Classification = Classification.public_api,
        tags: list[Label | str] | None = None,
        connections: list[Manifest] | None = None,
        forges: list[Label | str] | None = None,
        info: Info | None = None,
        github: Github | None = None,
        analytics: PostHogSource | None = None,
        metrics: Metrics | None = None,
        cicd: CICD | None = None,
        tests: Tests | None = None,
        dependencies: Dependencies | None = None,
        performance: Performance | None = None,
        other_diagram: OtherDiagram | None = None,
        description: Markdown | str | None = None,
        version: Version | str | None = None,
        language: ProgrammingLanguage | None = None,
        frameworks: list[WebFramework] | None = None,
        deployment: DeploymentTarget | None = None,
        owner_team: Label | str | None = None,
        auth_type: AuthType | None = None,
        monitoring: MonitoringType | None = None,
        tracing: TracingType | None = None,
        log_aggregation: LogAggregationType | None = None,
        iac_tool: IaCToolType | None = None,
        max_replicas: int | None = None,
        min_replicas: int | None = None,
        docs: list[Url | str] | None = None,
        links: list[Link] | None = None,
        repo_url: Url | str | None = None,
        cicd_tool: CICDToolType | str | None = None,
        **extra: Never,
    ) -> None:
        """Create a manifest, accepting common section fields as init-only sugar."""
        data: dict[str, Any] = {
            "name": name,
            "icon": icon,
            "classification": classification,
        }
        data.update(extra)
        optional_fields = {
            "manifest_id": manifest_id,
            "tags": tags,
            "connections": connections,
            "forges": forges,
            "analytics": analytics,
            "metrics": metrics,
            "tests": tests,
            "dependencies": dependencies,
            "performance": performance,
            "other_diagram": other_diagram,
        }
        data.update(
            {
                field_name: value
                for field_name, value in optional_fields.items()
                if value is not None
            }
        )

        info_updates = {
            "description": description,
            "version": version,
            "language": language,
            "frameworks": frameworks,
            "deployment": deployment,
            "owner_team": owner_team,
            "auth_type": auth_type,
            "monitoring": monitoring,
            "tracing": tracing,
            "log_aggregation": log_aggregation,
            "iac_tool": iac_tool,
            "max_replicas": max_replicas,
            "min_replicas": min_replicas,
            "docs": docs,
            "links": links,
        }
        info_updates = {
            field_name: value for field_name, value in info_updates.items() if value is not None
        }
        if info_updates:
            base_info = info.model_dump(mode="python", by_alias=False) if info else {}
            data["info"] = {**base_info, **info_updates}
        elif info is not None:
            data["info"] = info

        if repo_url is not None:
            base_github = github.model_dump(mode="python", by_alias=False) if github else {}
            data["github"] = {**base_github, "repo_url": repo_url}
        elif github is not None:
            data["github"] = github

        if cicd_tool is not None:
            base_cicd = cicd.model_dump(mode="python", by_alias=False) if cicd else {}
            data["cicd"] = {**base_cicd, "platform": cicd_tool}
        elif cicd is not None:
            data["cicd"] = cicd

        super().__init__(**data)

    @field_serializer("classification")
    def _serialize_classification(self, value: Classification) -> str:
        return value.value

    def framework_names(self) -> list[str]:
        """Return the framework identifier strings for each selected framework."""
        return [framework.framework for framework in self.info.frameworks]

    @override
    def dict(
        self,
        *,
        mode: str = "json",
        include: IncEx | None = None,
        exclude: IncEx | None = None,
        context: Any | None = None,
        by_alias: bool | None = None,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        exclude_computed_fields: bool = False,
        round_trip: bool = False,
        warnings: bool | Literal["none", "warn", "error"] = True,
        fallback: Callable[[Any], Any] | None = None,
        serialize_as_any: bool = False,
        polymorphic_serialization: bool | None = None,
    ) -> builtins.dict[str, Any]:
        """Return a JSON-friendly model dump."""
        return super().model_dump(
            mode=mode,
            include=include,
            exclude=exclude,
            context=context,
            by_alias=True if by_alias is None else by_alias,
            exclude_unset=exclude_unset,
            exclude_defaults=exclude_defaults,
            exclude_none=exclude_none,
            exclude_computed_fields=exclude_computed_fields,
            round_trip=round_trip,
            warnings=warnings,
            fallback=fallback,
            serialize_as_any=serialize_as_any,
            polymorphic_serialization=polymorphic_serialization,
        )

    def to_dict(self) -> builtins.dict[str, Any]:
        """Return a plain dict (JSON-safe) representation of this manifest."""
        import json

        return json.loads(self.model_dump_json(by_alias=True))

    def __str__(self) -> str:
        """Return a string representation of the manifest."""
        return f"<{self.name} ({self.info.language.value}) -> {self.info.deployment.id}>"


_MANIFEST_QUERY_SECTION_MODELS: dict[str, type[BaseModel]] = {
    "info": Info,
    "github": Github,
    "metrics": Metrics,
    "cicd": CICD,
    "tests": Tests,
    "dependencies": Dependencies,
    "performance": Performance,
    "other_diagram": OtherDiagram,
}

_MANIFEST_QUERY_FIELD_ALIASES: dict[str, str] = {
    "description": "info.description",
    "version": "info.version",
    "language": "info.language",
    "frameworks": "info.frameworks",
    "deployment": "info.deployment",
    "owner_team": "info.owner_team",
    "ownerTeam": "info.owner_team",
    "auth_type": "info.auth_type",
    "authType": "info.auth_type",
    "monitoring": "info.monitoring",
    "tracing": "info.tracing",
    "log_aggregation": "info.log_aggregation",
    "logAggregation": "info.log_aggregation",
    "iac_tool": "info.iac_tool",
    "iacTool": "info.iac_tool",
    "max_replicas": "info.max_replicas",
    "maxReplicas": "info.max_replicas",
    "min_replicas": "info.min_replicas",
    "minReplicas": "info.min_replicas",
    "docs": "info.docs",
    "links": "info.links",
    "repo_url": "github.repo_url",
    "repoUrl": "github.repo_url",
    "cicd_tool": "cicd.platform",
    "cicdTool": "cicd.platform",
    "otherDiagram": "other_diagram",
}


def _model_field_name(model: type[BaseModel], field_path_part: str) -> str | None:
    """Return the Python field name for a field name or serialization alias."""
    if field_path_part in model.model_fields:
        return field_path_part

    for field_name, field_info in model.model_fields.items():
        if field_info.serialization_alias == field_path_part:
            return field_name

    return None


def _normalize_manifest_query_path(path: str) -> str:
    """Normalize a query field path to Manifest model field names."""
    cleaned_path = path.replace("__", ".").strip(".")
    if not cleaned_path:
        msg = "Manifest query field paths cannot be empty"
        raise ValueError(msg)

    if cleaned_path in _MANIFEST_QUERY_FIELD_ALIASES:
        return _MANIFEST_QUERY_FIELD_ALIASES[cleaned_path]

    parts = cleaned_path.split(".")
    first_part = parts[0]
    if len(parts) == 1:
        top_level_field = _model_field_name(Manifest, first_part)
        if top_level_field is not None:
            return top_level_field

        section_matches = [
            f"{section_name}.{field_name}"
            for section_name, section_model in _MANIFEST_QUERY_SECTION_MODELS.items()
            if (field_name := _model_field_name(section_model, first_part)) is not None
        ]
        if len(section_matches) == 1:
            return section_matches[0]

        msg = f"Unknown or ambiguous Manifest query field: {path!r}"
        raise ValueError(msg)

    top_level_field = _model_field_name(Manifest, first_part)
    if top_level_field is None:
        msg = f"Unknown Manifest query field: {path!r}"
        raise ValueError(msg)

    if top_level_field not in _MANIFEST_QUERY_SECTION_MODELS:
        msg = f"Manifest query field {top_level_field!r} does not support nested paths"
        raise ValueError(msg)

    section_model = _MANIFEST_QUERY_SECTION_MODELS[top_level_field]
    normalized_parts = [top_level_field]
    for nested_part in parts[1:]:
        nested_field = _model_field_name(section_model, nested_part)
        if nested_field is None:
            msg = f"Unknown Manifest query field: {path!r}"
            raise ValueError(msg)
        normalized_parts.append(nested_field)

    return ".".join(normalized_parts)


def _flatten_manifest_query_where(
    where: Mapping[str, object],
    prefix: str = "",
) -> dict[str, object]:
    """Flatten nested query dictionaries into dot-separated field paths."""
    flattened: dict[str, object] = {}
    for raw_key, value in where.items():
        key = raw_key.replace("__", ".")
        path = f"{prefix}.{key}" if prefix else key
        if isinstance(value, Mapping):
            nested_value = {
                str(nested_key): nested_item for nested_key, nested_item in value.items()
            }
            flattened.update(_flatten_manifest_query_where(nested_value, path))
        else:
            flattened[path] = value

    return flattened


def _manifest_query_value(value: object) -> object:
    """Normalize values for stable query comparison."""
    if isinstance(value, Enum):
        return str(value)
    if isinstance(value, BaseModel):
        return value.model_dump(mode="json", by_alias=True)
    if isinstance(value, Mapping):
        return {str(key): _manifest_query_value(item) for key, item in value.items()}
    if isinstance(value, Sequence) and not isinstance(value, str):
        return [_manifest_query_value(item) for item in value]
    return (
        str(value)
        if isinstance(value, (Label, Markdown, Url, SemVer, CalendarVersion, IncrementalVersion))
        else value
    )


def _manifest_query_matches(actual: object, expected: object) -> bool:
    """Return whether a Manifest field value satisfies a query condition."""
    normalized_actual = _manifest_query_value(actual)
    normalized_expected = _manifest_query_value(expected)

    if isinstance(normalized_actual, list):
        if isinstance(normalized_expected, list):
            return all(expected_item in normalized_actual for expected_item in normalized_expected)
        return normalized_expected in normalized_actual

    return normalized_actual == normalized_expected


def _manifest_query_path_value(manifest: Manifest, path: str) -> object:
    """Read a normalized dot-path from a Manifest instance."""
    current: object = manifest
    for part in path.split("."):
        if current is None:
            return _QUERY_MISSING
        if isinstance(current, BaseModel):
            current = getattr(current, part, _QUERY_MISSING)
            continue
        if isinstance(current, Mapping):
            current_mapping = {str(key): value for key, value in current.items()}
            current = current_mapping.get(part, _QUERY_MISSING)
            continue
        current = getattr(current, part, _QUERY_MISSING)

    return current


class ManifestQuery(BaseModel):
    """Filter Manifests by a subset of their Pydantic field values."""

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )

    where: dict[str, object] = Field(
        default_factory=dict,
        description=(
            "Manifest field filters. Keys may be dot paths such as `info.language`, "
            "Python-safe paths such as `info__language`, or common field shorthands "
            "such as `language` and `repo_url`."
        ),
    )

    def __init__(
        self,
        where: Mapping[str, object] | None = None,
        **fields: object,
    ) -> None:
        """Create a query from a where mapping and/or keyword field shorthands."""
        flattened = _flatten_manifest_query_where(where or {})
        flattened.update(_flatten_manifest_query_where(fields))
        normalized = {
            _normalize_manifest_query_path(path): _manifest_query_value(value)
            for path, value in flattened.items()
        }
        super().__init__(where=normalized)

    def matches(self, manifest: Manifest) -> bool:
        """Return whether *manifest* satisfies every query field condition."""
        for path, expected in self.where.items():
            actual = _manifest_query_path_value(manifest, path)
            if actual is _QUERY_MISSING or not _manifest_query_matches(actual, expected):
                return False

        return True

    def filter(self, manifests: Sequence[Manifest]) -> list[Manifest]:
        """Return the manifests that match this query, preserving input order."""
        return [manifest for manifest in manifests if self.matches(manifest)]


class Diagram(BaseModel):
    """A high-level Scryr diagram composed from selected manifests."""

    Query: ClassVar[type[ManifestQuery]] = ManifestQuery

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )

    name: Label | str = Field(default=Label(""), description="Display name for the diagram")
    description: Markdown | str = Field(
        default=Markdown(""),
        description="Short description of the diagram scope",
    )
    manifests: list[Manifest] = Field(
        default_factory=list,
        description="Manifest blocks included in this diagram",
    )
    query: ManifestQuery | None = Field(
        default=None,
        description=(
            "Optional Manifest query used as an alternative to explicitly listing manifests"
        ),
    )

    @model_validator(mode="after")
    def _validate_manifest_selection(self) -> Diagram:
        """Ensure diagrams use either explicit manifests or a query, not both."""
        if self.query is not None and self.manifests:
            msg = "Diagram accepts either manifests or query, not both"
            raise ValueError(msg)
        return self

    def resolved_manifests(self, manifest_pool: Sequence[Manifest]) -> list[Manifest]:
        """Return explicit manifests or query-selected manifests from *manifest_pool*."""
        if self.query is None:
            return self.manifests
        return self.query.filter(manifest_pool)

    @override
    def dict(
        self,
        *,
        mode: str = "json",
        include: IncEx | None = None,
        exclude: IncEx | None = None,
        context: Any | None = None,
        by_alias: bool | None = None,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        exclude_computed_fields: bool = False,
        round_trip: bool = False,
        warnings: bool | Literal["none", "warn", "error"] = True,
        fallback: Callable[[Any], Any] | None = None,
        serialize_as_any: bool = False,
        polymorphic_serialization: bool | None = None,
    ) -> builtins.dict[str, Any]:
        """Return a JSON-friendly model dump."""
        return super().model_dump(
            mode=mode,
            include=include,
            exclude=exclude,
            context=context,
            by_alias=True if by_alias is None else by_alias,
            exclude_unset=exclude_unset,
            exclude_defaults=exclude_defaults,
            exclude_none=exclude_none,
            exclude_computed_fields=exclude_computed_fields,
            round_trip=round_trip,
            warnings=warnings,
            fallback=fallback,
            serialize_as_any=serialize_as_any,
            polymorphic_serialization=polymorphic_serialization,
        )

    def to_dict(
        self,
        manifest_pool: Sequence[Manifest] | None = None,
    ) -> builtins.dict[str, Any]:
        """Return a plain dict (JSON-safe) representation of this diagram."""
        import json

        data = json.loads(self.model_dump_json(by_alias=True))
        if manifest_pool is not None:
            data["manifests"] = [
                manifest.to_dict() for manifest in self.resolved_manifests(manifest_pool)
            ]
        return data
