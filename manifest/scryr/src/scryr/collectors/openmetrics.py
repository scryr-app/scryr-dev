"""Direct CLI scraping of an application's OpenMetrics exposition endpoint."""

from typing import Annotated, Literal
from urllib.parse import urlsplit

from pydantic import Field, field_validator

from .common import EnvRef, Identifier, NonEmpty, Schedule, _CollectorConfig, _Config, polling

type MetricName = Annotated[str, Field(pattern=r"^[A-Za-z_:][A-Za-z0-9_:]*$", max_length=256)]


class CounterRate(_Config):
    """Calculate reset-aware counter rate over successive collected samples."""

    kind: Literal["counter_rate"] = "counter_rate"
    metric: MetricName
    title: NonEmpty | None = None


class GaugeSeries(_Config):
    """Display the latest gauge value and collected trend."""

    kind: Literal["gauge"] = "gauge"
    metric: MetricName
    title: NonEmpty | None = None


class HistogramPercentile(_Config):
    """Estimate a percentile from compatible histogram buckets, never mean quantiles."""

    kind: Literal["histogram_percentile"] = "histogram_percentile"
    metric: MetricName
    title: NonEmpty | None = None
    percentile: float = Field(gt=0, le=100, allow_inf_nan=False)


type MetricSelection = Annotated[
    CounterRate | GaugeSeries | HistogramPercentile, Field(discriminator="kind")
]


class OpenMetricsCollector(_CollectorConfig):
    """Scrape bounded typed samples without a provider server or query language."""

    kind: Literal["openmetrics"] = "openmetrics"
    id: Identifier = "openmetrics"
    endpoint: NonEmpty
    series: list[MetricSelection] = Field(default_factory=list, max_length=200)
    max_series: int = Field(default=200, ge=1, le=10000)
    auth: EnvRef | None = None
    schedule: Schedule = Field(default_factory=lambda: polling(30))

    @field_validator("endpoint")
    @classmethod
    def _http_endpoint(cls, value: str) -> str:
        parsed = urlsplit(value)
        if (
            parsed.scheme not in {"http", "https"}
            or not parsed.hostname
            or parsed.username
            or parsed.password
            or parsed.fragment
        ):
            msg = "Metrics endpoint must be HTTP(S) without embedded credentials or a fragment"
            raise ValueError(msg)
        return value
