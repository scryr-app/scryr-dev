"""Rust test collection using cargo-nextest and generated report settings."""

from typing import Literal

from pydantic import Field

from .common import Identifier, NonEmpty, ProjectPath, _CollectorConfig


class NextestCollector(_CollectorConfig):
    """Run nextest with private generated JUnit configuration."""

    kind: Literal["nextest"] = "nextest"
    id: Identifier = "nextest"
    directory: ProjectPath = "."
    args: list[NonEmpty] = Field(default_factory=list)
