"""Package and license inventory declarations for Syft."""

from typing import Literal

from pydantic import Field

from .common import Identifier, ProjectPath, Schedule, _CollectorConfig


class SyftInventoryCollector(_CollectorConfig):
    """Inventory declared project files and retain an SBOM for dependent collectors."""

    kind: Literal["syft_inventory"] = "syft_inventory"
    id: Identifier = "syft_inventory"
    directory: ProjectPath = "."
    exclude: list[ProjectPath] = Field(default_factory=list)
    schedule: Schedule = Field(
        default_factory=lambda: Schedule(
            startup=True,
            watch=[
                "**/uv.lock",
                "**/poetry.lock",
                "**/requirements*.txt",
                "**/package-lock.json",
                "**/pnpm-lock.yaml",
                "**/yarn.lock",
                "**/Cargo.lock",
                "**/go.sum",
                "**/pyproject.toml",
                "**/package.json",
                "**/Cargo.toml",
            ],
        )
    )
