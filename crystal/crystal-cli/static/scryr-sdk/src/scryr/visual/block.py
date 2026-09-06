"""Visual block representation for isometric flowcharts."""

from __future__ import annotations

from typing import TYPE_CHECKING

from pydantic import BaseModel, ConfigDict, Field

if TYPE_CHECKING:
    from scryr.visual.block_id import BlockID
    from scryr.visual.color import Color
    from scryr.visual.renderable import Renderable


class VisualBlock(BaseModel):
    """A visual block in an isometric flowchart.

    Represents a 3D block with three visible faces:
    - Top face: Always a label (required)
    - Right side face: Any generic visual component (optional)
    - Left side face: Any generic visual component (optional)
    """

    model_config = ConfigDict(extra="forbid", validate_default=True)

    block_id: BlockID | None = Field(
        default=None,
        description="Unique identifier for the block (defaults to top_face label if not provided)",
    )

    top_face: Renderable = Field(
        ...,
        description="Label displayed on the top face of the block (required)",
    )

    right_side_face: Renderable | None = Field(
        default=None,
        description="Visual component for the right side face (optional)",
    )

    left_side_face: Renderable | None = Field(
        default=None,
        description="Visual component for the left side face (optional)",
    )

    color: Color | None = Field(
        default=None,
        description="Color of the block (optional)",
    )

    relationships: list[BlockID] = Field(
        default_factory=list,
        description="List of relationship identifiers to other blocks",
    )
