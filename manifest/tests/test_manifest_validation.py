"""Tests for Manifest field validation, constraints, and extra field rejection."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from scryr.manifest import Forge, InfoManifestSection, Link, Manifest


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


def test_manifest_enforces_replica_minimum_constraints() -> None:
    """Replica fields enforce minimum value constraints."""
    with pytest.raises(ValidationError, match="greater than or equal to 1"):
        Manifest(name="BadReplicas", info=InfoManifestSection(max_replicas=0))

    with pytest.raises(ValidationError, match="greater than or equal to 1"):
        Manifest(name="BadReplicas", info=InfoManifestSection(min_replicas=0))


def test_manifest_default_list_fields_are_isolated_per_instance() -> None:
    """List defaults are isolated; mutating one instance does not affect another."""
    first = Manifest(name="First")
    second = Manifest(name="Second")

    first.connections.append("OtherService")
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
        info=InfoManifestSection(docs=["https://api.example.com", "Architecture RFC"]),
    )

    data = manifest.to_dict()
    assert data["info"]["docs"] == ["https://api.example.com", "Architecture RFC"]


def test_manifest_rejects_min_replicas_above_max_replicas() -> None:
    """Replica bounds must remain ordered."""
    with pytest.raises(
        ValidationError, match="min_replicas must be less than or equal to max_replicas"
    ):
        Manifest(name="ReplicaBounds", info=InfoManifestSection(min_replicas=3, max_replicas=2))
