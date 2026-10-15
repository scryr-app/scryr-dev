"""Declarative, server-resolved runtime metric sources."""

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, model_validator


class CredentialRef(BaseModel):
    """Name of an organization-scoped server connection; never a secret."""

    model_config = ConfigDict(extra="forbid")
    name: str = Field(min_length=1, max_length=128)


class PrometheusSource(BaseModel):
    """Query a Prometheus-compatible backend only when a diagram opens."""

    model_config = ConfigDict(extra="forbid", populate_by_name=True)
    kind: Literal["prometheus"] = "prometheus"
    query_endpoint: str | None = Field(default=None, alias="queryEndpoint")
    credentials: CredentialRef
    dashboard_url: str | None = Field(default=None, alias="dashboardUrl")
    environment: str = "production"
    refresh: Literal["on_diagram_load"] = "on_diagram_load"
    window: int = Field(default=900, ge=60, le=86400, description="Window seconds")
    step: int = Field(default=60, ge=15, le=3600, description="Sample step seconds")
    cache_ttl: int = Field(default=60, ge=1, le=3600, alias="cacheTtl")
    ingestion_delay: int = Field(default=120, ge=0, le=3600, alias="ingestionDelay")
    queries: dict[str, str] = Field(min_length=1, max_length=12)
    units: dict[str, str] = Field(default_factory=dict)

    @model_validator(mode="after")
    def validate_bounds(self) -> PrometheusSource:
        """Bound sample volume and require readable query names and expressions."""
        if self.window // self.step > 1440:
            msg = "Metric windows may contain at most 1440 intervals"
            raise ValueError(msg)
        if any(
            not key or len(key) > 64 or not expr.strip() or len(expr) > 4096
            for key, expr in self.queries.items()
        ):
            msg = "Queries require names up to 64 characters and expressions up to 4096"
            raise ValueError(msg)
        if not self.units.keys() <= self.queries.keys():
            msg = "Units must refer to configured queries"
            raise ValueError(msg)
        return self


class PostHogSource(PrometheusSource):
    """Bounded HogQL aggregate queries resolved only when opening a diagram.

    Queries return one numeric cell. {start} and {end} are Unix seconds;
    {environment} is a server-escaped SQL string literal.
    """

    kind: Literal["posthog"] = "posthog"
    project_id: int | None = Field(default=None, alias="projectId", gt=0)
    window: int = Field(default=86400, ge=60, le=86400)
    labels: dict[str, str] = Field(default_factory=dict)

    @model_validator(mode="after")
    def validate_analytics(self) -> PostHogSource:
        """Keep query labels bounded and every aggregate scoped to its window."""
        if not self.labels.keys() <= self.queries.keys() or any(
            not value.strip() or len(value) > 80 for value in self.labels.values()
        ):
            msg = "Labels must name configured queries and contain 1..80 characters"
            raise ValueError(msg)
        if any(
            not all(token in query for token in ("{start}", "{end}", "{environment}"))
            for query in self.queries.values()
        ):
            msg = "PostHog queries require start, end, and environment placeholders"
            raise ValueError(msg)
        return self
