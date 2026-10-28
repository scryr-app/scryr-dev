"""Description Markdown typing and serialized compatibility."""

import pytest
from pydantic import ValidationError

from scryr import Info, Manifest
from scryr.types import Markdown


@pytest.mark.parametrize(
    "description",
    [
        "Plain descriptions remain supported.",
        Markdown("## API\n\n**Public** endpoint with ~~legacy~~ routes.\n\n- [x] Migrated"),
        "| Route | Status |\n| --- | --- |\n| `/health` | Ready |",
    ],
)
def test_description_is_markdown_and_round_trips_as_text(description: Markdown | str) -> None:
    """Plain source text and explicit Markdown preserve content on the wire."""
    info = Info.model_validate({"description": description})
    assert isinstance(info.description, Markdown)
    assert info.description == description

    manifest = Manifest(name="API", info=info)
    payload = manifest.to_dict()
    assert payload["info"]["description"] == description
    restored = Manifest.model_validate_json(manifest.model_dump_json())
    assert isinstance(restored.info.description, Markdown)
    assert restored.info.description == description


def test_description_default_and_shorthand_are_markdown() -> None:
    """Empty defaults and constructor shorthand normalize to the semantic type."""
    assert isinstance(Info().description, Markdown)
    assert Info().description == ""
    shorthand = Manifest(name="API", description="**Healthy**")
    assert isinstance(shorthand.info.description, Markdown)
    assert shorthand.info.description == "**Healthy**"


def test_description_schema_advertises_markdown() -> None:
    """The schema exposes one Markdown string rather than an untyped alternative."""
    description = Info.model_json_schema()["properties"]["description"]
    assert description["type"] == "string"
    assert description["contentMediaType"] == "text/markdown"
    assert "anyOf" not in description


@pytest.mark.parametrize("description", [None, 42, True, [], {"text": "not text"}])
def test_description_rejects_non_text(description: object) -> None:
    """Do not silently stringify structured or numeric values as Markdown."""
    with pytest.raises(ValidationError, match="description must be Markdown text"):
        Info.model_validate({"description": description})
