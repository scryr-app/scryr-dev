"""Typed version models for SemVer, Calendar, and Incremental versioning."""

from __future__ import annotations

import re
from dataclasses import dataclass
from functools import total_ordering
from typing import Final, overload

SEMVER_RE: Final[re.Pattern[str]] = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?"
    r"(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
)
CALENDAR_RE: Final[re.Pattern[str]] = re.compile(r"^(\d{4})\.(0?[1-9]|1[0-2])$")
INCREMENTAL_RE: Final[re.Pattern[str]] = re.compile(r"^(0|[1-9]\d*)$")


@dataclass(frozen=True, slots=True, init=False)
@total_ordering
class SemVer:
    """Semantic version with proper precedence semantics.

    Build metadata is preserved in the value but ignored for ordering.
    """

    major: int
    minor: int
    patch: int
    prerelease: tuple[str, ...] = ()
    build: tuple[str, ...] = ()

    @overload
    def __init__(self, value: str, /) -> None: ...

    @overload
    def __init__(
        self,
        major: int,
        minor: int,
        patch: int,
        prerelease: tuple[str, ...] = (),
        build: tuple[str, ...] = (),
    ) -> None: ...

    def __init__(
        self,
        major: str | int,
        minor: int | None = None,
        patch: int | None = None,
        prerelease: tuple[str, ...] = (),
        build: tuple[str, ...] = (),
    ) -> None:
        """Create SemVer from either a full string or numeric components."""
        if isinstance(major, str):
            if minor is not None or patch is not None or prerelease or build:
                msg = "String SemVer input does not accept additional components"
                raise TypeError(msg)
            parsed = self.parse(major)
            object.__setattr__(self, "major", parsed.major)
            object.__setattr__(self, "minor", parsed.minor)
            object.__setattr__(self, "patch", parsed.patch)
            object.__setattr__(self, "prerelease", parsed.prerelease)
            object.__setattr__(self, "build", parsed.build)
            return

        if minor is None or patch is None:
            msg = "SemVer numeric input requires major, minor, and patch"
            raise TypeError(msg)

        object.__setattr__(self, "major", major)
        object.__setattr__(self, "minor", minor)
        object.__setattr__(self, "patch", patch)
        object.__setattr__(self, "prerelease", prerelease)
        object.__setattr__(self, "build", build)

    @classmethod
    def parse(cls, value: str) -> SemVer:
        """Parse and validate a semantic version string."""
        normalized = _strip_optional_v_prefix(value)
        match = SEMVER_RE.fullmatch(normalized)
        if match is None:
            msg = (
                "Invalid SemVer. Expected MAJOR.MINOR.PATCH (optionally prefixed with 'v') "
                "with optional "
                "-PRERELEASE and +BUILD metadata"
            )
            raise ValueError(msg)

        major = int(match.group(1))
        minor = int(match.group(2))
        patch = int(match.group(3))

        prerelease_raw = match.group(4)
        build_raw = match.group(5)
        prerelease = tuple(prerelease_raw.split(".")) if prerelease_raw else ()
        build = tuple(build_raw.split(".")) if build_raw else ()

        # SemVer requires numeric prerelease identifiers to not include leading zeros.
        for identifier in prerelease:
            if identifier.isdigit() and len(identifier) > 1 and identifier.startswith("0"):
                msg = "Numeric prerelease identifiers must not contain leading zeros"
                raise ValueError(msg)

        return cls(major, minor, patch, prerelease, build)

    def __str__(self) -> str:
        """Return the canonical string form."""
        base = f"{self.major}.{self.minor}.{self.patch}"
        prerelease = f"-{'.'.join(self.prerelease)}" if self.prerelease else ""
        build = f"+{'.'.join(self.build)}" if self.build else ""
        return f"{base}{prerelease}{build}"

    def __lt__(self, other: object) -> bool:
        """Compare semantic versions using SemVer precedence rules."""
        if not isinstance(other, SemVer):
            return NotImplemented

        core_self = (self.major, self.minor, self.patch)
        core_other = (other.major, other.minor, other.patch)
        if core_self != core_other:
            return core_self < core_other

        # A version without prerelease has higher precedence than one with prerelease.
        if not self.prerelease and other.prerelease:
            return False
        if self.prerelease and not other.prerelease:
            return True
        if not self.prerelease and not other.prerelease:
            return False

        return _compare_prerelease(self.prerelease, other.prerelease) < 0

    def __eq__(self, other: object) -> bool:
        """SemVer equality by precedence (build metadata does not affect equality)."""
        if not isinstance(other, SemVer):
            return False
        return (
            self.major,
            self.minor,
            self.patch,
            self.prerelease,
        ) == (
            other.major,
            other.minor,
            other.patch,
            other.prerelease,
        )

    def __hash__(self) -> int:
        """Hash using fields involved in equality/precedence semantics."""
        return hash((self.major, self.minor, self.patch, self.prerelease))


@dataclass(frozen=True, slots=True, init=False)
class CalendarVersion:
    """Calendar version in YEAR.MONTH format."""

    year: int
    month: int

    @overload
    def __init__(self, value: str, /) -> None: ...

    @overload
    def __init__(self, year: int, month: int) -> None: ...

    def __init__(self, year: str | int, month: int | None = None) -> None:
        """Create calendar version from either YEAR.MONTH or integer components."""
        if isinstance(year, str):
            if month is not None:
                msg = "String calendar input does not accept a separate month"
                raise TypeError(msg)
            parsed = self.parse(year)
            object.__setattr__(self, "year", parsed.year)
            object.__setattr__(self, "month", parsed.month)
            return

        if month is None:
            msg = "Calendar numeric input requires both year and month"
            raise TypeError(msg)

        object.__setattr__(self, "year", year)
        object.__setattr__(self, "month", month)

    @classmethod
    def parse(cls, value: str) -> CalendarVersion:
        """Parse and validate a calendar version string."""
        normalized = _strip_optional_v_prefix(value)
        match = CALENDAR_RE.fullmatch(normalized)
        if match is None:
            msg = "Invalid calendar version. Expected YEAR.MONTH (optionally prefixed with 'v')"
            raise ValueError(msg)
        return cls(int(match.group(1)), int(match.group(2)))

    def __str__(self) -> str:
        """Return canonical calendar format."""
        return f"{self.year}.{self.month}"


@dataclass(frozen=True, slots=True, init=False)
class IncrementalVersion:
    """Incremental numeric version (single integer)."""

    number: int

    @overload
    def __init__(self, value: str, /) -> None: ...

    @overload
    def __init__(self, value: int, /) -> None: ...

    def __init__(self, number: str | int) -> None:
        """Create incremental version from either a string or an integer."""
        if isinstance(number, str):
            parsed = self.parse(number)
            object.__setattr__(self, "number", parsed.number)
            return

        if number < 0:
            msg = "Invalid incremental version. Expected a non-negative integer"
            raise ValueError(msg)
        object.__setattr__(self, "number", number)

    @classmethod
    def parse(cls, value: str) -> IncrementalVersion:
        """Parse and validate an incremental version string."""
        normalized = _strip_optional_v_prefix(value)
        match = INCREMENTAL_RE.fullmatch(normalized)
        if match is None:
            msg = (
                "Invalid incremental version. Expected a non-negative integer "
                "(optionally prefixed with 'v')"
            )
            raise ValueError(msg)
        return cls(int(match.group(1)))

    def __str__(self) -> str:
        """Return canonical incremental format."""
        return str(self.number)


Version = SemVer | CalendarVersion | IncrementalVersion
Incremental = IncrementalVersion


def parse_version(value: str) -> Version:
    """Parse version using the supported formats in specificity order."""
    for parser in (SemVer.parse, CalendarVersion.parse, IncrementalVersion.parse):
        try:
            return parser(value)
        except ValueError:
            continue

    msg = (
        "Unsupported version format. Supported: SemVer (MAJOR.MINOR.PATCH), "
        "Calendar (YEAR.MONTH), Incremental (Numbering). "
        f"Input: {value!r}."
    )
    raise ValueError(msg)


def _compare_prerelease(left: tuple[str, ...], right: tuple[str, ...]) -> int:
    """Compare prerelease identifiers according to SemVer precedence rules."""
    for left_item, right_item in zip(left, right, strict=False):
        if left_item == right_item:
            continue

        left_is_num = left_item.isdigit()
        right_is_num = right_item.isdigit()

        if left_is_num and right_is_num:
            return -1 if int(left_item) < int(right_item) else 1

        if left_is_num and not right_is_num:
            return -1
        if not left_is_num and right_is_num:
            return 1

        return -1 if left_item < right_item else 1

    if len(left) == len(right):
        return 0
    return -1 if len(left) < len(right) else 1


def _strip_optional_v_prefix(value: str) -> str:
    """Strip a single leading v prefix when present."""
    return value.removeprefix("v")
