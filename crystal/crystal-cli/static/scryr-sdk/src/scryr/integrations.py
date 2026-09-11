"""Provider-specific declarations; credentials are resolved outside the manifest."""

from datetime import timedelta
from enum import StrEnum
from typing import Annotated, ClassVar, Literal, Self, TypedDict

from pydantic import (
    BaseModel,
    ConfigDict,
    Field,
    ModelWrapValidatorHandler,
    PrivateAttr,
    computed_field,
    model_validator,
)
from pydantic.alias_generators import to_camel

from .github import GithubActionsLog  # noqa: TC001 - Pydantic resolves this at runtime.
from .metrics_source import CredentialRef, PostHogSource, PrometheusSource


class CardCategory(StrEnum):
    """Categories shared by integrations and their concrete views."""

    repository = "repository"
    metrics = "metrics"
    cicd = "cicd"
    tests = "tests"
    dependencies = "dependencies"
    performance = "performance"
    uptime = "uptime"
    analytics = "analytics"


class MetricUnit(StrEnum):
    """Units supported by metric displays."""

    percent = "%"
    milliseconds = "ms"
    seconds = "s"
    bytes = "bytes"
    requests_per_second = "req/s"
    count = "count"
    ratio = "ratio"


class _Model(BaseModel):
    """Strict declaration serialization shared by providers."""

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        alias_generator=to_camel,
        validate_default=True,
        validate_assignment=True,
    )


class _Declaration(_Model):
    """Identity is assigned by the source loader from a public variable binding."""

    _declaration_name: str = PrivateAttr(default="")

    @model_validator(mode="wrap")
    @classmethod
    def restore_declaration(cls, value: object, handler: ModelWrapValidatorHandler[Self]) -> Self:
        """Restore derived wire metadata while keeping it out of constructor fields."""
        if not isinstance(value, dict):
            return handler(value)
        fields = dict(value)
        name = fields.pop("id", "")
        categories = (
            fields.pop("categories", None) if "categories" in cls.model_computed_fields else None
        )
        source = fields.pop("source", None) if "source" in cls.model_computed_fields else None
        result = handler(fields)
        if not isinstance(name, str) or (name and not name.isidentifier()):
            msg = "Declaration IDs must be Python variable names"
            raise ValueError(msg)
        result._declaration_name = name
        if (
            categories is not None
            and result.model_dump(mode="json").get("categories") != categories
        ):
            msg = "Integration categories are determined by its provider"
            raise ValueError(msg)
        if source is not None and result.model_dump(mode="json", by_alias=True).get("source") != (
            PostHogSource.model_validate(source)
            if source.get("kind") == "posthog"
            else PrometheusSource.model_validate(source)
        ).model_dump(mode="json", by_alias=True):
            msg = "Derived query source does not match the typed declaration"
            raise ValueError(msg)
        return result

    @computed_field
    @property
    def id(self) -> str:
        """Return the source variable name without accepting an explicit ID."""
        return self._declaration_name


class GrafanaAuthentication(_Declaration):
    """Reference a TOML table containing a Grafana username and token."""

    kind: Literal["grafana"] = "grafana"


class PostHogAuthentication(_Declaration):
    """Reference a TOML table containing a PostHog API key."""

    kind: Literal["posthog"] = "posthog"


class GitHubAuthentication(_Declaration):
    """Reference a TOML table containing a GitHub token."""

    kind: Literal["github"] = "github"


type Endpoint = Annotated[str, Field(pattern=r"^https://[^\s]+$", max_length=2048)]
type Expression = Annotated[str, Field(min_length=1, max_length=4096)]


class _Integration(_Declaration):
    """Provider capabilities are derived, never selected by a string in source."""

    _categories: ClassVar[tuple[CardCategory, ...]] = ()

    @computed_field
    @property
    def categories(self) -> tuple[CardCategory, ...]:
        """Return the categories this provider supports."""
        return self._categories


class Grafana(_Integration):
    """A reusable Prometheus-compatible Grafana connection."""

    kind: Literal["grafana"] = "grafana"
    authentication: GrafanaAuthentication
    endpoint: Endpoint
    dashboard_url: Endpoint | None = None
    _categories: ClassVar[tuple[CardCategory, ...]] = (
        CardCategory.metrics,
        CardCategory.performance,
        CardCategory.uptime,
    )


class PostHog(_Integration):
    """A reusable PostHog project connection."""

    kind: Literal["posthog"] = "posthog"
    authentication: PostHogAuthentication
    endpoint: Endpoint = "https://us.posthog.com"
    project_id: int = Field(gt=0)
    dashboard_url: Endpoint | None = None
    _categories: ClassVar[tuple[CardCategory, ...]] = (
        CardCategory.performance,
        CardCategory.analytics,
    )


class GitHubActions(_Integration):
    """A repository's Actions integration, independent of its displayed pipelines."""

    kind: Literal["github_actions"] = "github_actions"
    repository: str = Field(pattern=r"^[^/\s]+/[^/\s]+$")
    authentication: GitHubAuthentication | None = None
    _categories: ClassVar[tuple[CardCategory, ...]] = (CardCategory.cicd,)


class PrometheusQuery(_Model):
    """One PromQL expression and its display unit."""

    expression: Expression
    unit: MetricUnit = MetricUnit.count


class PostHogQuery(_Model):
    """One numeric HogQL aggregate, explicitly bounded by time and environment."""

    expression: Expression
    unit: MetricUnit = MetricUnit.count
    label: str | None = Field(default=None, min_length=1, max_length=80)

    @model_validator(mode="after")
    def validate_scope(self) -> PostHogQuery:
        """Require placeholders that the server substitutes safely."""
        if not all(t in self.expression for t in ("{start}", "{end}", "{environment}")):
            msg = "PostHog queries require start, end, and environment placeholders"
            raise ValueError(msg)
        return self


class GrafanaPerformanceQueries(_Model):
    """Resource performance measurements supported by GrafanaPerformance."""

    cpu_current: PrometheusQuery | None = None
    cpu_avg: PrometheusQuery | None = None
    cpu_peak: PrometheusQuery | None = None
    memory_usage: PrometheusQuery | None = None


class GrafanaMetricsQueries(_Model):
    """Request measurements supported by GrafanaMetrics."""

    request_rate: PrometheusQuery | None = None
    response_time_p50: PrometheusQuery | None = None
    response_time_p95: PrometheusQuery | None = None
    response_time_p99: PrometheusQuery | None = None
    error_rate: PrometheusQuery | None = None


class GrafanaUptimeQueries(_Model):
    """Availability measurements supported by GrafanaUptime."""

    uptime: PrometheusQuery | None = None


class PostHogPerformanceQueries(_Model):
    """Browser performance measurements supported by PostHogPerformance."""

    largest_contentful_paint: PostHogQuery | None = None
    interaction_to_next_paint: PostHogQuery | None = None
    cumulative_layout_shift: PostHogQuery | None = None
    response_time_p50: PostHogQuery | None = None
    response_time_p95: PostHogQuery | None = None
    response_time_p99: PostHogQuery | None = None


class PostHogAnalyticsQueries(_Model):
    """Product measurements supported by PostHogAnalytics."""

    page_views: PostHogQuery | None = None
    active_users: PostHogQuery | None = None
    sessions: PostHogQuery | None = None
    conversions: PostHogQuery | None = None


class MetricWindow(_Model):
    """Bounded, on-diagram-load sampling; durations use timedelta in source."""

    duration: timedelta = timedelta(minutes=15)
    step: timedelta = timedelta(minutes=1)
    cache_ttl: timedelta = timedelta(minutes=1)
    ingestion_delay: timedelta = timedelta(minutes=2)

    @model_validator(mode="after")
    def validate_bounds(self) -> MetricWindow:
        """Match server sampling limits and reject fractional seconds."""
        bounds = (
            (self.duration, 60, 86400),
            (self.step, 15, 3600),
            (self.cache_ttl, 1, 3600),
            (self.ingestion_delay, 0, 3600),
        )
        if any(not low <= d.total_seconds() <= high or d.microseconds for d, low, high in bounds):
            msg = "Metric durations must be whole seconds within the supported bounds"
            raise ValueError(msg)
        if self.duration / self.step > 1440:
            msg = "Metric windows may contain at most 1440 intervals"
            raise ValueError(msg)
        return self


class _SourceOptions(TypedDict):
    """Typed transport options shared by both supported query protocols."""

    credentials: CredentialRef
    environment: str
    window: int
    step: int
    cache_ttl: int
    ingestion_delay: int
    queries: dict[str, str]
    units: dict[str, str]


class _MetricView(_Declaration):
    """Common settings for concrete metric views."""

    environment: str = Field(default="production", min_length=1, max_length=128)
    window: MetricWindow = Field(default_factory=MetricWindow)

    def _source_options(self, queries: _Model, authentication: _Declaration) -> _SourceOptions:
        """Translate typed query fields to the bounded runtime protocol."""
        entries = {
            to_camel(k): getattr(queries, k)
            for k in type(queries).model_fields
            if getattr(queries, k) is not None
        }
        return {
            "credentials": CredentialRef(name=authentication.id or "unbound"),
            "environment": self.environment,
            "window": int(self.window.duration.total_seconds()),
            "step": int(self.window.step.total_seconds()),
            "cache_ttl": int(self.window.cache_ttl.total_seconds()),
            "ingestion_delay": int(self.window.ingestion_delay.total_seconds()),
            "queries": {k: q.expression for k, q in entries.items()},
            "units": {k: q.unit.value for k, q in entries.items()},
        }


class _GrafanaView(_MetricView):
    """A view whose connection can only be Grafana."""

    integration: Grafana

    def _grafana_source(self, queries: _Model) -> PrometheusSource | None:
        """Produce a runtime source only when queries have been configured."""
        options = self._source_options(queries, self.integration.authentication)
        if not options["queries"]:
            return None
        return PrometheusSource(
            query_endpoint=self.integration.endpoint,
            dashboard_url=self.integration.dashboard_url,
            **options,
        )


class GrafanaPerformance(_GrafanaView):
    """Display CPU and memory observations from Grafana."""

    kind: Literal["grafana_performance"] = "grafana_performance"
    category: Literal[CardCategory.performance] = CardCategory.performance
    title: str = "Grafana Performance"
    queries: GrafanaPerformanceQueries = Field(default_factory=GrafanaPerformanceQueries)

    @computed_field
    @property
    def source(self) -> PrometheusSource | None:
        """Return the server query declaration."""
        return self._grafana_source(self.queries)


class GrafanaMetrics(_GrafanaView):
    """Display request rates, latency, and errors from Grafana."""

    kind: Literal["grafana_metrics"] = "grafana_metrics"
    category: Literal[CardCategory.metrics] = CardCategory.metrics
    title: str = "Grafana Metrics"
    queries: GrafanaMetricsQueries = Field(default_factory=GrafanaMetricsQueries)

    @computed_field
    @property
    def source(self) -> PrometheusSource | None:
        """Return the server query declaration."""
        return self._grafana_source(self.queries)


class GrafanaUptime(_GrafanaView):
    """Display availability from a declared Grafana query."""

    kind: Literal["grafana_uptime"] = "grafana_uptime"
    category: Literal[CardCategory.uptime] = CardCategory.uptime
    title: str = "Grafana Uptime"
    queries: GrafanaUptimeQueries = Field(default_factory=GrafanaUptimeQueries)

    @computed_field
    @property
    def source(self) -> PrometheusSource | None:
        """Return the server query declaration."""
        return self._grafana_source(self.queries)


class _PostHogView(_MetricView):
    """A view whose connection can only be PostHog."""

    integration: PostHog
    window: MetricWindow = Field(default_factory=lambda: MetricWindow(duration=timedelta(days=1)))

    def _posthog_source(self, queries: _Model) -> PostHogSource | None:
        """Produce an aggregate source scoped to the integration's project."""
        options = self._source_options(queries, self.integration.authentication)
        if not options["queries"]:
            return None
        return PostHogSource(
            query_endpoint=self.integration.endpoint,
            project_id=self.integration.project_id,
            dashboard_url=self.integration.dashboard_url,
            labels={
                to_camel(k): q.label
                for k in type(queries).model_fields
                if isinstance(q := getattr(queries, k), PostHogQuery) and q.label
            },
            **options,
        )


class PostHogPerformance(_PostHogView):
    """Display browser performance aggregates from PostHog."""

    kind: Literal["posthog_performance"] = "posthog_performance"
    category: Literal[CardCategory.performance] = CardCategory.performance
    title: str = "PostHog Performance"
    queries: PostHogPerformanceQueries = Field(default_factory=PostHogPerformanceQueries)

    @computed_field
    @property
    def source(self) -> PostHogSource | None:
        """Return the server query declaration."""
        return self._posthog_source(self.queries)


class PostHogAnalytics(_PostHogView):
    """Display product usage aggregates from PostHog."""

    kind: Literal["posthog_analytics"] = "posthog_analytics"
    category: Literal[CardCategory.analytics] = CardCategory.analytics
    title: str = "PostHog Analytics"
    queries: PostHogAnalyticsQueries = Field(default_factory=PostHogAnalyticsQueries)

    @computed_field
    @property
    def source(self) -> PostHogSource | None:
        """Return the server query declaration."""
        return self._posthog_source(self.queries)


class GitHubActionsPipeline(_Declaration):
    """Display reported Actions runs for a repository and optional workflow/branch."""

    kind: Literal["github_actions_pipeline"] = "github_actions_pipeline"
    category: Literal[CardCategory.cicd] = CardCategory.cicd
    title: str = "GitHub Actions"
    integration: GitHubActions
    observations: GithubActionsLog | None = None
    workflow_id: int | None = Field(default=None, gt=0)
    branch: str | None = Field(default=None, min_length=1)


type Integration = Annotated[Grafana | PostHog | GitHubActions, Field(discriminator="kind")]
type IntegrationView = Annotated[
    GrafanaPerformance
    | GrafanaMetrics
    | GrafanaUptime
    | PostHogPerformance
    | PostHogAnalytics
    | GitHubActionsPipeline,
    Field(discriminator="kind"),
]
