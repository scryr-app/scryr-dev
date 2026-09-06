"""Renderable content types for visual elements."""

from __future__ import annotations

from enum import StrEnum

from pydantic import BaseModel, ConfigDict, Field


class RenderableType(StrEnum):
    """Types of renderable content."""

    text = "text"
    markdown = "markdown"
    svg = "svg"


class Renderable(BaseModel):
    """A renderable piece of content that can be displayed in various formats.

    Can represent text, markdown, or SVG content for use in block faces,
    labels, descriptions, and other visual elements.
    """

    model_config = ConfigDict(extra="forbid", validate_default=True)

    content: str = Field(
        ...,
        description="The content to render",
    )

    type: RenderableType = Field(
        default=RenderableType.text,
        description="The type of content (text, markdown, or svg)",
    )

    def __str__(self) -> str:
        """Return the content as a string."""
        return self.content
