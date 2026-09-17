"""JavaScript test collection using the project's Vitest installation."""

from typing import Literal

from pydantic import Field

from .common import Identifier, NonEmpty, ProjectPath, _CollectorConfig


class VitestCollector(_CollectorConfig):
    """Run Vitest once and arrange a fresh JUnit report."""

    kind: Literal["vitest"] = "vitest"
    id: Identifier = "vitest"
    directory: ProjectPath = "."
    args: list[NonEmpty] = Field(default_factory=list)
