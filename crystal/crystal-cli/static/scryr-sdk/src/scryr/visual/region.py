"""Visual region representation for grouping blocks in isometric flowcharts."""

from __future__ import annotations

from typing import TYPE_CHECKING

from pydantic import BaseModel, ConfigDict, Field

if TYPE_CHECKING:
    from scryr.visual.block_id import BlockID
    from scryr.visual.color import Color
    from scryr.visual.renderable import Renderable


class Region(BaseModel):
    """A visual region grouping multiple blocks in an isometric flowchart.

    Represents a logical grouping or container for related blocks.
    """

    model_config = ConfigDict(extra="forbid", validate_default=True)

    label: Renderable = Field(
        ...,
        description="Label for the region (required)",
    )

    description: Renderable | None = Field(
        default=None,
        description="Description of what this region represents (optional)",
    )

    block_ids: list[BlockID] = Field(
        default_factory=list,
        description="List of block identifiers contained in this region",
    )

    color: Color | None = Field(
        default=None,
        description="Background or border color for the region (optional)",
    )

    border_style: str = Field(
        default="solid",
        description="Border style (solid, dashed, dotted, etc.)",
    )

    border_thickness: float = Field(
        default=1.0,
        description="Thickness of the region border",
        gt=0,
    )
