"""Tests for typed version parsing and ordering."""

import pytest

from scryr.types import (
    CalendarVersion,
    Incremental,
    IncrementalVersion,
    SemVer,
)
from scryr.version import parse_version


def test_version_accepts_semver_calendar_and_incremental() -> None:
    """parse_version returns strongly typed version models."""
    semver = parse_version("1.2.3")
    calendar = parse_version("2026.5")
    incremental = parse_version("42")

    assert isinstance(semver, SemVer)
    assert isinstance(calendar, CalendarVersion)
    assert isinstance(incremental, IncrementalVersion)

    assert str(semver) == "1.2.3"
    assert str(calendar) == "2026.5"
    assert str(incremental) == "42"


def test_semver_rejects_leading_zeros_in_core() -> None:
    """SemVer core identifiers cannot have leading zeros."""
    with pytest.raises(ValueError, match="Unsupported version format"):
        parse_version("01.2.3")


@pytest.mark.parametrize("value", ["1.2.a", "1.2"])
def test_invalid_semver_formats_are_rejected(value: str) -> None:
    """Invalid SemVer values should not validate."""
    with pytest.raises(ValueError, match="Unsupported version format"):
        parse_version(value)


def test_semver_prerelease_precedence() -> None:
    """Pre-release versions are lower precedence than release versions."""
    alpha = SemVer.parse("1.0.0-alpha")
    release = SemVer.parse("1.0.0")

    assert alpha < release


def test_semver_build_metadata_does_not_affect_ordering() -> None:
    """Build metadata must not change version precedence."""
    left = SemVer.parse("1.0.0+build.1")
    right = SemVer.parse("1.0.0+build.2")

    assert left == right
    assert not (left < right)
    assert not (right < left)


@pytest.mark.parametrize("value", ["1", "01", "001"])
def test_incremental_rejects_leading_zeros(value: str) -> None:
    """Incremental numbering rejects leading zeros except zero itself."""
    if value == "1":
        assert str(parse_version(value)) == "1"
        return

    with pytest.raises(ValueError, match="Unsupported version format"):
        parse_version(value)


def test_versions_support_direct_string_or_int_constructors() -> None:
    """Version models can be directly instantiated from string/int wire values."""
    assert str(SemVer("7.3.0")) == "7.3.0"
    assert str(SemVer("v7.3.0")) == "7.3.0"
    assert str(CalendarVersion("2026.3")) == "2026.3"
    assert str(CalendarVersion("v2026.3")) == "2026.3"
    assert str(IncrementalVersion("123")) == "123"
    assert str(IncrementalVersion("v123")) == "123"
    assert str(Incremental(123)) == "123"


def test_parse_version_accepts_v_prefix() -> None:
    """parse_version accepts optional v-prefixed string versions."""
    assert str(parse_version("v1.2.3")) == "1.2.3"
    assert str(parse_version("v2026.3")) == "2026.3"
    assert str(parse_version("v123")) == "123"
