"""Tests for Manifest field validation, constraints, and extra field rejection."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from scryr.manifest import Forge, Info, Link, Manifest


def test_manifest_rejects_unknown_fields() -> None:
    """Manifest forbids extra unknown fields via Pydantic config."""
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        Manifest.model_validate({"name": "Strict", "unknown_field": "nope"})


def test_link_rejects_unknown_fields() -> None:
    """Link model forbids unknown fields via Pydantic config."""
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        Link.model_validate(
            {
                "site_name": "Docs",
                "http_url": "https://example.com",
                "extra": "nope",
            }
        )


def test_forge_rejects_unknown_top_level_fields() -> None:
    """Forge forbids unknown Scryr wrapper fields while modeled mise entries stay typed."""
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        Forge.model_validate({"name": "Strict", "unknown_field": "nope"})


def test_manifest_default_list_fields_are_isolated_per_instance() -> None:
    """List defaults are isolated; mutating one instance does not affect another."""
    first = Manifest(name="First")
    second = Manifest(name="Second")

    first.connections.append(Manifest(name="OtherService"))
    first.tags.append("tagged")
    first.info.docs.append("https://docs.example.com")
    first.forges.append("node")

    assert second.connections == []
    assert second.tags == []
    assert second.info.docs == []
    assert second.forges == []


def test_manifest_docs_field_accepts_url_and_plain_text_values() -> None:
    """Docs field accepts both URL-like values and plain strings."""
    manifest = Manifest(
        name="DocsTypes",
        info=Info(docs=["https://api.example.com", "Architecture RFC"]),
    )

    data = manifest.to_dict()
    assert data["info"]["docs"] == ["https://api.example.com", "Architecture RFC"]


@pytest.mark.parametrize("field", ["min_replicas", "max_replicas", "minReplicas", "maxReplicas"])
def test_info_rejects_removed_replica_fields(field: str) -> None:
    """Removed replica fields are rejected in source and serialized manifests."""
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        Manifest.model_validate({"name": "Service", "info": {field: 2}})


def test_info_schema_and_serialization_omit_replica_fields() -> None:
    """Default manifests and published schemas no longer advertise replicas."""
    removed = {"min_replicas", "max_replicas", "minReplicas", "maxReplicas"}
    assert removed.isdisjoint(Manifest(name="Service").to_dict()["info"])
    assert removed.isdisjoint(Info.model_json_schema()["properties"])
