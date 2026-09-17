"""File-only integrations; declaring these never starts a test runner."""

from typing import Literal

from pydantic import Field, model_validator

from .common import Identifier, NonEmpty, ProjectPath, Schedule, _CollectorConfig


class _ReportConfig(_CollectorConfig):
    """Common artifact scope for concrete file formats."""

    files: list[ProjectPath] = Field(min_length=1)
    suite: NonEmpty = "default"

    @model_validator(mode="after")
    def _watch_files_by_default(self) -> _ReportConfig:
        if "schedule" not in self.model_fields_set:
            self.schedule = Schedule(startup=True, watch=list(self.files))
        return self


class JUnitReportCollector(_ReportConfig):
    """Read JUnit suite counts and cases from existing reports."""

    kind: Literal["junit"] = "junit"
    id: Identifier = "junit"


class LcovCoverageCollector(_ReportConfig):
    """Read executable and covered line counts from LCOV files."""

    kind: Literal["lcov"] = "lcov"
    id: Identifier = "lcov"


class CoberturaCoverageCollector(_ReportConfig):
    """Read line coverage from Cobertura XML files."""

    kind: Literal["cobertura"] = "cobertura"
    id: Identifier = "cobertura"
