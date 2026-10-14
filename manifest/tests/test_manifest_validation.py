"""Tests for Manifest field validation, constraints, and extra field rejection."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from scryr.manifest import Forge, Manifest


def test_manifest_rejects_unknown_fields() -> None:
    """Manifest forbids extra unknown fields via Pydantic config."""
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        Manifest.model_validate({"name": "Strict", "unknown_field": "nope"})


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
    first.forges.append("node")

    assert second.connections == []
    assert second.tags == []
    assert second.forges == []


@pytest.mark.parametrize(
    "field",
    [
        "info",
        "github",
        "analytics",
        "metrics",
        "cicd",
        "tests",
        "dependencies",
        "performance",
        "otherDiagram",
    ],
)
def test_manifest_rejects_removed_cards(field: str) -> None:
    """Old card payloads cannot silently re-enter the public manifest contract."""
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        Manifest.model_validate({"name": "Service", field: {}})
