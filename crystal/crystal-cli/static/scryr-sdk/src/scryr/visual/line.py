"""Visual line representation for connecting blocks in isometric flowcharts."""

from __future__ import annotations

from enum import StrEnum
from typing import TYPE_CHECKING

from pydantic import BaseModel, ConfigDict, Field

if TYPE_CHECKING:
    from scryr.visual.block_id import BlockID
    from scryr.visual.color import Color
    from scryr.visual.renderable import Renderable


class LineSegmentType(StrEnum):
    """Types of line segments for visual connections."""

    solid = "solid"
    dashed = "dashed"
    dotted = "dotted"
    right_arrow = "right_arrow"
    left_arrow = "left_arrow"
    bidirectional_arrow = "bidirectional_arrow"


class Line(BaseModel):
    """A visual line connecting two blocks in an isometric flowchart.

    Represents a connection between blocks with a label and visual style.
    """

    model_config = ConfigDict(extra="forbid", validate_default=True)

    from_block: BlockID = Field(
        ...,
        description="Identifier of the source block",
    )

    to_block: BlockID = Field(
        ...,
        description="Identifier of the destination block",
    )

    label: Renderable | None = Field(
        default=None,
        description="Label displayed on or near the line (optional)",
    )

    description: Renderable | None = Field(
        default=None,
        description="Description of the relationship (optional)",
    )

    segment_type: LineSegmentType = Field(
        default=LineSegmentType.solid,
        description="Visual style of the line segment",
    )

    color: Color | None = Field(
        default=None,
        description="Color of the line (optional)",
    )

    thickness: float = Field(
        default=1.0,
        description="Thickness of the line",
        gt=0,
    )

    relationship_type: str | None = Field(
        default=None,
        description="Type of relationship this line represents (optional)",
    )
