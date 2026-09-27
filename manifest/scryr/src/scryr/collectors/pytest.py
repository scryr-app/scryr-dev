"""Python test collection through pytest and a fresh JUnit artifact."""

from typing import Literal

from pydantic import Field

from .common import Identifier, NonEmpty, ProjectPath, _CollectorConfig


class PytestCollector(_CollectorConfig):
    """Run local pytest and arrange a fresh, owned JUnit report."""

    kind: Literal["pytest"] = "pytest"
    id: Identifier = "pytest"
    directory: ProjectPath = "."
    paths: list[ProjectPath] = Field(default_factory=lambda: ["tests"], min_length=1)
    args: list[NonEmpty] = Field(default_factory=list)
