"""Typed, inert integration declarations for Scryr's six local evidence cards."""

from typing import Annotated

from pydantic import Field

from .checks import BiomeCheckCollector, CargoClippyCollector, MiseTaskCollector, RuffCheckCollector
from .common import Command, EnvRef, LicensePolicy, SbomRef, Schedule, ToolRequirement
from .docker import DockerStatsCollector
from .git import GitStatusCollector
from .github import GitHubActionsCollector, GitHubPullRequestCollector
from .grant import GrantLicenseCollector
from .grype import GrypeScanCollector
from .hyperfine import HyperfineBenchmarkCollector
from .nextest import NextestCollector
from .openmetrics import CounterRate, GaugeSeries, HistogramPercentile, OpenMetricsCollector
from .pytest import PytestCollector
from .reports import CoberturaCoverageCollector, JUnitReportCollector, LcovCoverageCollector
from .syft import SyftInventoryCollector
from .vitest import VitestCollector

type RepositoryCollector = Annotated[
    GitStatusCollector | GitHubPullRequestCollector | GitHubActionsCollector,
    Field(discriminator="kind"),
]
type CheckCollector = Annotated[
    RuffCheckCollector | BiomeCheckCollector | CargoClippyCollector | MiseTaskCollector,
    Field(discriminator="kind"),
]
type MetricCollector = Annotated[
    OpenMetricsCollector | DockerStatsCollector, Field(discriminator="kind")
]
type TestCollector = Annotated[
    PytestCollector
    | VitestCollector
    | NextestCollector
    | JUnitReportCollector
    | LcovCoverageCollector
    | CoberturaCoverageCollector,
    Field(discriminator="kind"),
]
type DependencyCollector = Annotated[
    SyftInventoryCollector | GrypeScanCollector | GrantLicenseCollector,
    Field(discriminator="kind"),
]
type PerformanceCollector = HyperfineBenchmarkCollector

__all__ = [
    "BiomeCheckCollector",
    "CargoClippyCollector",
    "CheckCollector",
    "CoberturaCoverageCollector",
    "Command",
    "CounterRate",
    "DependencyCollector",
    "DockerStatsCollector",
    "EnvRef",
    "GaugeSeries",
    "GitHubActionsCollector",
    "GitHubPullRequestCollector",
    "GitStatusCollector",
    "GrantLicenseCollector",
    "GrypeScanCollector",
    "HistogramPercentile",
    "HyperfineBenchmarkCollector",
    "JUnitReportCollector",
    "LcovCoverageCollector",
    "LicensePolicy",
    "MetricCollector",
    "MiseTaskCollector",
    "NextestCollector",
    "OpenMetricsCollector",
    "PerformanceCollector",
    "PytestCollector",
    "RepositoryCollector",
    "RuffCheckCollector",
    "SbomRef",
    "Schedule",
    "SyftInventoryCollector",
    "TestCollector",
    "ToolRequirement",
    "VitestCollector",
]
