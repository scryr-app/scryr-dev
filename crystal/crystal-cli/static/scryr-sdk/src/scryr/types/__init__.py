"""Centralized Scryr type exports used by Manifest and consumers.

This package keeps the existing domain structure (`saas`, `version`, `text`) in
place while providing a single import surface for all Manifest-related types.
"""

from __future__ import annotations

from scryr.saas.cicd_tool import CICDToolType
from scryr.saas.classification import Classification
from scryr.saas.data_persistence import IaCToolType
from scryr.saas.deployment_target import DeploymentTarget
from scryr.saas.environment import AuthType
from scryr.saas.interface import InterfaceType
from scryr.saas.programming_languages import ProgrammingLanguage
from scryr.saas.telemetry import LogAggregationType, MonitoringType, TracingType
from scryr.saas.web_frameworks import WebFramework
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
    "AuthType",
    "CICDToolType",
    "CalendarVersion",
    "Classification",
    "DeploymentTarget",
    "IaCToolType",
    "Incremental",
    "IncrementalVersion",
    "InterfaceType",
    "Label",
    "LogAggregationType",
    "Markdown",
    "MonitoringType",
    "ProgrammingLanguage",
    "SemVer",
    "TracingType",
    "Url",
    "Version",
    "WebFramework",
    "parse_version",
]
