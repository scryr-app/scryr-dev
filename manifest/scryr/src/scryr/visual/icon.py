"""Icon validation utilities."""

import re
from typing import Annotated

from pydantic import AfterValidator

_ICON_EXTENSIONS = (".png", ".svg", ".ico", ".jpg", ".jpeg", ".gif")
_emoji_pattern = re.compile(
    "[\U0001f600-\U0001f64f\U0001f300-\U0001f5ff\U0001f680-\U0001f6ff\U0001f1e0-\U0001f1ff]+",
    flags=re.UNICODE,
)


def _validate_icon(v: str) -> str:
    if _emoji_pattern.match(v) or v.lower().endswith(_ICON_EXTENSIONS):
        return v
    msg = f"Invalid icon: '{v}' — must be emoji or one of {_ICON_EXTENSIONS}"
    raise ValueError(msg)


Icon = Annotated[str, AfterValidator(_validate_icon)]
