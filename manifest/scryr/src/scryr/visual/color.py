"""Color type for visual elements."""

from __future__ import annotations

from enum import StrEnum

from pydantic import BaseModel, ConfigDict, Field, field_validator

# Tailwind CSS color palette hex values
TAILWIND_COLORS = {
    # Slate
    "slate": "#64748b",
    # Gray
    "gray": "#6b7280",
    # Zinc
    "zinc": "#71717a",
    # Neutral
    "neutral": "#737373",
    # Stone
    "stone": "#78716c",
    # Red
    "red": "#ef4444",
    # Orange
    "orange": "#f97316",
    # Amber
    "amber": "#f59e0b",
    # Yellow
    "yellow": "#eab308",
    # Lime
    "lime": "#84cc16",
    # Green
    "green": "#22c55e",
    # Emerald
    "emerald": "#10b981",
    # Teal
    "teal": "#14b8a6",
    # Cyan
    "cyan": "#06b6d4",
    # Sky
    "sky": "#0ea5e9",
    # Blue
    "blue": "#3b82f6",
    # Indigo
    "indigo": "#6366f1",
    # Violet
    "violet": "#8b5cf6",
    # Purple
    "purple": "#a855f7",
    # Fuchsia
    "fuchsia": "#d946ef",
    # Pink
    "pink": "#ec4899",
    # Rose
    "rose": "#f43f5e",
    # White
    "white": "#ffffff",
    # Black
    "black": "#000000",
}


class ColorName(StrEnum):
    """Named colors from Tailwind CSS palette."""

    slate = "slate"
    gray = "gray"
    zinc = "zinc"
    neutral = "neutral"
    stone = "stone"
    red = "red"
    orange = "orange"
    amber = "amber"
    yellow = "yellow"
    lime = "lime"
    green = "green"
    emerald = "emerald"
    teal = "teal"
    cyan = "cyan"
    sky = "sky"
    blue = "blue"
    indigo = "indigo"
    violet = "violet"
    purple = "purple"
    fuchsia = "fuchsia"
    pink = "pink"
    rose = "rose"
    white = "white"
    black = "black"


class Color(BaseModel):
    """A color representation that can be either a named color or a hex code.

    Accepts either:
    - A named color from Tailwind CSS palette (e.g., "red", "blue", "emerald")
    - Hex color codes in formats: #RRGGBB or #RRGGBBAA
    """

    model_config = ConfigDict(extra="forbid", validate_default=True)

    value: ColorName | str = Field(
        ...,
        description="Either a Tailwind color name or hex code (#RRGGBB or #RRGGBBAA)",
    )

    @field_validator("value", mode="before")
    @classmethod
    def validate_color(cls, v: ColorName | str) -> ColorName | str:
        """Validate that the color is either a known color name or valid hex code."""
        if isinstance(v, ColorName):
            return v

        # Check if it's a color name string
        try:
            return ColorName(v.lower())
        except ValueError, AttributeError:
            pass

        # Otherwise, validate as hex code
        v = str(v).strip()
        if not v.startswith("#"):
            msg = "Color must be a Tailwind color name or start with #"
            raise ValueError(msg)

        hex_part = v[1:]
        if len(hex_part) not in (6, 8):
            msg = "Hex color must be #RRGGBB or #RRGGBBAA"
            raise ValueError(msg)

        try:
            int(hex_part, 16)
        except ValueError:
            msg = "Hex color must contain valid hexadecimal characters"
            raise ValueError(msg) from None

        return v.upper()

    def render(self) -> str:
        """Return the color in hex format.

        If value is a named color, returns its Tailwind hex value.
        If value is already a hex code, returns it as-is.
        """
        value_str = str(self.value).lower()
        if value_str in TAILWIND_COLORS:
            return TAILWIND_COLORS[value_str]
        return str(self.value)

    def __str__(self) -> str:
        """Return the color as a string."""
        return str(self.value)
