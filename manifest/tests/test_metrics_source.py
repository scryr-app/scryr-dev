"""Declarative analytics configuration contracts."""

import pytest


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
