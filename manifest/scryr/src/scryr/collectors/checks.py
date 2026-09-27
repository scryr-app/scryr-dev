"""Concrete local lint and task integrations; manual unless explicitly scheduled."""

from typing import Literal

from pydantic import Field

from .common import Identifier, NonEmpty, ProjectPath, _CollectorConfig


class RuffCheckCollector(_CollectorConfig):
    """Run Ruff's structured diagnostics without applying fixes."""

    kind: Literal["ruff"] = "ruff"
    id: Identifier = "ruff"
    directory: ProjectPath = "."
    paths: list[ProjectPath] = Field(default_factory=lambda: ["."], min_length=1)


class BiomeCheckCollector(_CollectorConfig):
    """Run Biome's structured checks without applying fixes."""

    kind: Literal["biome"] = "biome"
    id: Identifier = "biome"
    directory: ProjectPath = "."
    paths: list[ProjectPath] = Field(default_factory=lambda: ["."], min_length=1)


class CargoClippyCollector(_CollectorConfig):
    """Run local Rust Clippy diagnostics."""

    kind: Literal["clippy"] = "clippy"
    id: Identifier = "clippy"
    directory: ProjectPath = "."
    all_targets: bool = True


class MiseTaskCollector(_CollectorConfig):
    """Execute one declared mise task as a local check, preserving its exit outcome."""

    kind: Literal["mise_task"] = "mise_task"
    id: Identifier = "mise_task"
    directory: ProjectPath = "."
    task: NonEmpty
    args: list[NonEmpty] = Field(default_factory=list)
    forge: NonEmpty | None = None
