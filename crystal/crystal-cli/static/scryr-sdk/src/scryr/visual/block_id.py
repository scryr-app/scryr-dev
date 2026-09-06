"""Block identifier type for referencing blocks."""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class BlockID(BaseModel):
    """A unique identifier for a visual block.

    Defaults to the main label of the block but can be customized.
    """

    model_config = ConfigDict(extra="forbid", validate_default=True)

    id: str = Field(
        ...,
        description="Unique identifier for the block",
    )

    def __str__(self) -> str:
        """Return the ID as a string."""
        return self.id

    def __hash__(self) -> int:
        """Make BlockID hashable for use in sets and dicts."""
        return hash(self.id)

    def __eq__(self, other: object) -> bool:
        """Compare BlockIDs by their ID string."""
        if isinstance(other, BlockID):
            return self.id == other.id
        return False
