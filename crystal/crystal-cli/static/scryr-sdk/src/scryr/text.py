"""Scryr semantic string types.

Both types are transparent ``str`` subclasses — they serialize to plain strings
in JSON and are fully compatible with any code that expects ``str``.  The only
purpose of the distinct types is to carry semantic meaning through the type
system and surface in ``manifest_types.json`` output.

``Label``
    A short single-line display string (names, team identifiers, tags, emoji).

``Markdown``
    A longer text field that may contain Markdown formatting (descriptions,
    release notes, README snippets).
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import pydantic_core
from pydantic_core import core_schema

if TYPE_CHECKING:
    from pydantic import GetCoreSchemaHandler, GetJsonSchemaHandler
    from pydantic.json_schema import JsonSchemaValue


def _make_str_subclass_schema(cls: type[str]) -> core_schema.CoreSchema:
    """Return a Pydantic v2 core schema that validates as ``str`` and coerces to *cls*."""

    def coerce(v: object) -> Any:  # noqa: ANN401
        if isinstance(v, str):
            return cls(v)
        # Accept URL-like objects (e.g. pydantic.HttpUrl) via str conversion
        try:
            return cls(str(v))
        except Exception as exc:
            msg = f"Expected str or string-like, got {type(v).__name__}"
            raise TypeError(msg) from exc

    return core_schema.no_info_plain_validator_function(
        coerce,
        serialization=core_schema.plain_serializer_function_ser_schema(str),
    )


class Label(str):
    """A short, single-line display label (name, team, tag, etc.)."""

    __slots__ = ()

    @classmethod
    def __get_pydantic_core_schema__(
        cls, _source_type: object, _handler: GetCoreSchemaHandler
    ) -> pydantic_core.CoreSchema:
        """Register Pydantic v2 validation schema for Label."""
        return _make_str_subclass_schema(cls)

    @classmethod
    def __get_pydantic_json_schema__(
        cls, _core_schema: core_schema.CoreSchema, _handler: GetJsonSchemaHandler
    ) -> JsonSchemaValue:
        """Emit ``{"type": "string"}`` in JSON Schema output."""
        return {"type": "string", "title": "Label"}


class Markdown(str):
    """A Markdown-formatted long-form text field (description, notes, etc.)."""

    __slots__ = ()

    @classmethod
    def __get_pydantic_core_schema__(
        cls, _source_type: object, _handler: GetCoreSchemaHandler
    ) -> pydantic_core.CoreSchema:
        """Register Pydantic v2 validation schema for Markdown."""
        return _make_str_subclass_schema(cls)

    @classmethod
    def __get_pydantic_json_schema__(
        cls, _core_schema: core_schema.CoreSchema, _handler: GetJsonSchemaHandler
    ) -> JsonSchemaValue:
        """Emit a ``{"type": "string", "contentMediaType": "text/markdown"}`` JSON Schema."""
        return {"type": "string", "contentMediaType": "text/markdown", "title": "Markdown"}


class Url(str):
    """A URL string field (documentation links, external references, etc.)."""

    __slots__ = ()

    @classmethod
    def __get_pydantic_core_schema__(
        cls, _source_type: object, _handler: GetCoreSchemaHandler
    ) -> pydantic_core.CoreSchema:
        """Register Pydantic v2 validation schema for Url."""
        return _make_str_subclass_schema(cls)

    @classmethod
    def __get_pydantic_json_schema__(
        cls, _core_schema: core_schema.CoreSchema, _handler: GetJsonSchemaHandler
    ) -> JsonSchemaValue:
        """Emit ``{"type": "string", "format": "uri"}`` in JSON Schema output."""
        return {"type": "string", "format": "uri", "title": "Url"}
