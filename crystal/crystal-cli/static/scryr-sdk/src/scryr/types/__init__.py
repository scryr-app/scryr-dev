"""Centralized Scryr type exports used by Manifest and consumers.

This package keeps the existing domain structure (`saas`, `version`, `text`) in
place while providing a single import surface for all Manifest-related types.
"""

from __future__ import annotations

from scryr.saas.classification import Classification
from scryr.saas.interface import InterfaceType
from scryr.text import Label, Markdown, Url
from scryr.version import (
    CalendarVersion,
    Incremental,
    IncrementalVersion,
    SemVer,
    Version,
    parse_version,
)

__all__ = [
    "CalendarVersion",
    "Classification",
    "Incremental",
    "IncrementalVersion",
    "InterfaceType",
    "Label",
    "Markdown",
    "SemVer",
    "Url",
    "Version",
    "parse_version",
]
