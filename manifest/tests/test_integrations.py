"""Concrete integration declarations, identity, and wire compatibility."""

from datetime import timedelta
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

import pytest
from pydantic import ValidationError

from scryr import (
    CardCategory,
    Grafana,
    GrafanaAuthentication,
    GrafanaPerformance,
    GrafanaPerformanceQueries,
    GrafanaUptime,
    Manifest,
    MetricWindow,
    PostHog,
    PostHogAuthentication,
    PostHogQuery,
    PrometheusQuery,
)
from scryr.runtime import load_manifest_module

SOURCE = """from scryr import (
    CardCategory, Diagram, Grafana, GrafanaAuthentication, GrafanaPerformance,
    GrafanaPerformanceQueries, GrafanaUptime, GrafanaUptimeQueries, Manifest,
    MetricUnit, PostHog, PostHogAuthentication, PostHogPerformance, PrometheusQuery,
)
grafana_authentication = GrafanaAuthentication()
posthog_authentication = PostHogAuthentication()
grafana = Grafana(authentication=grafana_authentication, endpoint="https://metrics.example.com")
posthog = PostHog(authentication=posthog_authentication, project_id=123)
grafana_performance = GrafanaPerformance(
    integration=grafana,
    queries=GrafanaPerformanceQueries(
        cpu_current=PrometheusQuery(expression="cpu", unit=MetricUnit.percent),
    ),
)
grafana_uptime = GrafanaUptime(
    integration=grafana,
    queries=GrafanaUptimeQueries(uptime=PrometheusQuery(expression="up", unit=MetricUnit.ratio)),
)
posthog_performance = PostHogPerformance(integration=posthog)
api = Manifest(
    name="API", integrations=[grafana, posthog],
    cards=[grafana_performance, posthog_performance, grafana_uptime],
    card_categories=[CardCategory.performance, CardCategory.uptime],
)
diagram = Diagram(name="Typed integrations", manifests=[api])
"""


def test_identity_and_multiple_provider_views_roundtrip(tmp_path: Path) -> None:
    """Distinct typed views retain variable IDs and query bindings through serialization."""
    path = tmp_path / "index.scry"
    path.write_text(SOURCE)
    module = load_manifest_module(path)
    manifest = module.api
    encoded = manifest.to_dict()
    assert manifest.manifest_id == "api"
    assert [c["id"] for c in encoded["cards"]] == [
        "grafana_performance",
        "posthog_performance",
        "grafana_uptime",
    ]
    assert [c["category"] for c in encoded["cards"]] == ["performance", "performance", "uptime"]
    assert encoded["cards"][0]["source"]["credentials"]["name"] == "grafana_authentication"
    assert encoded["cards"][0]["source"]["queries"] == {"cpuCurrent": "cpu"}
    assert encoded["cards"][2]["source"]["queries"] == {"uptime": "up"}
    assert encoded["cards"][1]["source"] is None
    assert "token" not in str(encoded)
    assert Manifest.model_validate(encoded).to_dict() == encoded


@pytest.mark.parametrize(
    "change",
    [
        "alias = grafana_performance",
        "api.cards.append(GrafanaPerformance(integration=grafana))",
        'api.manifest_id = "custom-id"',
    ],
)
def test_rejects_ambiguous_or_missing_variable_identity(tmp_path: Path, change: str) -> None:
    """Aliases and anonymous cards cannot silently change durable identities."""
    path = tmp_path / "index.scry"
    path.write_text(SOURCE + change + "\n")
    with pytest.raises(ValueError, match=r"Ambiguous|public variable|variable names"):
        load_manifest_module(path)


def test_provider_and_query_types_are_specific() -> None:
    """A provider's view rejects the wrong integration or metric fields."""
    posthog = PostHog(authentication=PostHogAuthentication(), project_id=1)
    with pytest.raises(ValidationError):
        GrafanaPerformance.model_validate({"integration": posthog})
    with pytest.raises(ValidationError):
        GrafanaPerformanceQueries.model_validate({"uptime": {"expression": "up"}})
    with pytest.raises(ValidationError):
        Grafana.model_validate(
            {"authentication": PostHogAuthentication(), "endpoint": "https://example.com"}
        )
    with pytest.raises(ValueError, match="placeholders"):
        PostHogQuery(expression="SELECT count() FROM events")


def test_manifest_membership_presence_and_independent_integrations() -> None:
    """Integration setup does not force a card, and each card requires membership."""
    grafana = Grafana(authentication=GrafanaAuthentication(), endpoint="https://example.com")
    performance = GrafanaPerformance(integration=grafana)
    uptime = GrafanaUptime(integration=grafana)
    assert Manifest(integrations=[grafana], cards=[]).cards == []
    assert Manifest(integrations=[grafana], cards=[performance, uptime]).cards == [
        performance,
        uptime,
    ]
    with pytest.raises(ValueError, match="included"):
        Manifest(cards=[performance])
    with pytest.raises(ValueError, match="once"):
        Manifest(integrations=[grafana], cards=[performance, performance])
    with pytest.raises(ValueError, match="unique"):
        Manifest(card_categories=[CardCategory.performance, CardCategory.performance])


def test_sampling_limits_and_nonempty_queries() -> None:
    """Typed durations and expressions preserve server-side query bounds."""
    with pytest.raises(ValueError, match="intervals"):
        MetricWindow(duration=timedelta(days=1), step=timedelta(seconds=15))
    with pytest.raises(ValueError, match="whole seconds"):
        MetricWindow(step=timedelta(milliseconds=15500))
    with pytest.raises(ValidationError):
        PrometheusQuery(expression="")


def test_computed_source_cannot_be_overridden(tmp_path: Path) -> None:
    """A wire payload cannot attach an unrelated source to a typed view."""
    path = tmp_path / "index.scry"
    path.write_text(SOURCE)
    encoded = load_manifest_module(path).api.to_dict()
    encoded["cards"][0]["source"]["queries"]["cpuCurrent"] = "different"
    with pytest.raises(ValueError, match="Derived query source"):
        Manifest.model_validate(encoded)
