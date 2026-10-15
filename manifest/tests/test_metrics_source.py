"""Declarative runtime metric configuration contracts."""

import pytest
from pydantic import ValidationError

from scryr import CredentialRef, Manifest, Metrics, PrometheusSource


def test_source_round_trip() -> None:
    """Source settings survive manifest generation without resolving a secret."""
    source = PrometheusSource(
        credentials=CredentialRef(name="grafana"), queries={"requestRate": "up"}
    )
    manifest = Manifest(name="API", manifest_id="api", metrics=Metrics(provider=source))
    encoded = manifest.to_dict()
    assert encoded["metrics"]["provider"]["cacheTtl"] == 60
    assert encoded["metrics"]["provider"]["credentials"] == {"name": "grafana"}
    restored = Manifest.model_validate(encoded)
    assert restored.metrics is not None
    assert restored.metrics.provider == source


def test_source_rejects_unbounded_queries_and_polling() -> None:
    """Only bounded, on-load queries are accepted."""
    for options in ({"refresh": "poll"}, {"window": 86400, "step": 15}, {"queries": {}}):
        with pytest.raises(ValidationError):
            PrometheusSource.model_validate(
                {
                    "credentials": {"name": "grafana"},
                    "queries": {"rate": "up"},
                    **options,
                }
            )


def test_posthog_roundtrip_and_required_filters() -> None:
    """Analytics config survives generation and requires explicit query bounds."""
    from scryr import CredentialRef, Manifest, PostHogSource

    query = (
        "SELECT count() FROM events WHERE timestamp >= toDateTime({start}) "
        "AND timestamp < toDateTime({end}) AND properties.environment = {environment}"
    )
    source = PostHogSource(
        credentials=CredentialRef(name="posthog"),
        queries={"views": query},
        labels={"views": "Views"},
    )
    manifest = Manifest(name="web", analytics=source)
    assert manifest.model_dump(mode="json", by_alias=True)["analytics"]["kind"] == "posthog"
    assert source.window == 86400
    with pytest.raises(ValueError, match="placeholders"):
        PostHogSource(
            credentials=CredentialRef(name="posthog"),
            queries={"views": "SELECT count() FROM events"},
        )
