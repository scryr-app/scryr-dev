"""Tests for Manifest version field dispatch and validation."""

from __future__ import annotations

from typing import Any

import pytest
from pydantic import ValidationError

from scryr.manifest import InfoManifestSection, Manifest
from scryr.types import CalendarVersion, IncrementalVersion, SemVer


def test_manifest_default_version_is_typed_semver() -> None:
    """Default Manifest version is parsed into a SemVer instance."""
    manifest = Manifest(name="Defaults")

    assert isinstance(manifest.info.version, SemVer)
    assert str(manifest.info.version) == "0.1.0"


def test_manifest_accepts_string_version_and_dispatches_constructor() -> None:
    """String version input is dispatched to the appropriate typed version constructor."""
    semver_manifest = Manifest(name="SemVerSvc", info=InfoManifestSection(version=SemVer("7.3.0")))
    calendar_manifest = Manifest(
        name="CalendarSvc",
        info=InfoManifestSection(version=CalendarVersion("2026.3")),
    )
    incremental_manifest = Manifest(
        name="IncrementalSvc",
        info=InfoManifestSection(version=IncrementalVersion("123")),
    )

    assert isinstance(semver_manifest.info.version, SemVer)
    assert isinstance(calendar_manifest.info.version, CalendarVersion)
    assert isinstance(incremental_manifest.info.version, IncrementalVersion)


def test_manifest_accepts_already_typed_version_instances() -> None:
    """Typed version models are preserved when passed to Manifest directly."""
    semver = SemVer("1.2.3")
    calendar = CalendarVersion("2026.12")
    incremental = IncrementalVersion(10)

    semver_manifest = Manifest(name="SemVerSvc", info=InfoManifestSection(version=semver))
    calendar_manifest = Manifest(name="CalendarSvc", info=InfoManifestSection(version=calendar))
    incremental_manifest = Manifest(
        name="IncrementalSvc",
        info=InfoManifestSection(version=incremental),
    )

    assert semver_manifest.info.version is semver
    assert calendar_manifest.info.version is calendar
    assert incremental_manifest.info.version is incremental


def test_manifest_rejects_invalid_version_type() -> None:
    """Manifest rejects unsupported version input types."""
    invalid_version: Any = 123
    with pytest.raises(TypeError, match="version must be a version string"):
        Manifest(name="BadVersionType", info=InfoManifestSection(version=invalid_version))


def test_manifest_rejects_invalid_version_string() -> None:
    """Manifest rejects unsupported version string formats."""
    with pytest.raises(ValidationError, match="Unsupported version format"):
        Manifest.model_validate({"name": "BadVersion", "info": {"version": "release-latest"}})
