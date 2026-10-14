"""Tests for Manifest dict() and to_dict() serialization output."""

from __future__ import annotations

import pytest

from scryr.manifest import (
    Diagram,
    Forge,
    ForgeEnvDirective,
    ForgeTask,
    ForgeTaskBatch,
    ForgeTaskCommand,
    ForgeTool,
    Manifest,
    ManifestQuery,
)
from scryr.types import (
    Classification,
)


def test_forge_serializes_typed_mise_toml_sections() -> None:
    """Forge models keep mise.toml sections typed while serializing to JSON-safe values."""
    forge = Forge(
        name="Node Forge",
        tools={
            "node": ForgeTool(version="22", postinstall="corepack enable"),
            "python": "3.12",
        },
        env={
            "NODE_ENV": "development",
            "SECRET": ForgeEnvDirective(value="token", redact=True),
        },
        vars={"e2e_args": "--headless"},
        tasks={
            "dev": ForgeTask(run="npm run dev", depends="install"),
            "ci": ForgeTask(
                run=[
                    ForgeTaskCommand(task="lint"),
                    ForgeTaskBatch(tasks=["test", "build"]),
                    "echo done",
                ],
            ),
        },
        settings={"experimental": True},
        plugins={"node": "https://github.com/mise-plugins/mise-node"},
        redactions=["*_TOKEN"],
    )

    data = forge.to_dict()
    mise_data = forge.to_mise_dict()

    assert data["tools"]["node"]["version"] == "22"
    assert data["env"]["SECRET"]["redact"] is True
    assert mise_data["tasks"]["ci"]["run"][0]["task"] == "lint"
    assert mise_data["tasks"]["ci"]["run"][1]["tasks"] == ["test", "build"]
    assert "name" not in mise_data


def test_manifest_dict_respects_model_dump_kwargs() -> None:
    """Dict should forward model_dump kwargs rather than ignore them."""
    manifest = Manifest(name="IncludeOnly")

    data = manifest.dict(include={"name"})

    assert data == {"name": "IncludeOnly"}


def test_manifest_does_not_default_icon_from_classification() -> None:
    """Manifest leaves icon empty when no icon is explicitly set."""
    manifest = Manifest(
        name="Worker",
        classification=Classification.worker,
    )

    assert manifest.icon == ""


def test_manifest_preserves_explicit_icon() -> None:
    """Manifest serializes explicitly supplied icons."""
    manifest = Manifest(
        name="Worker",
        icon="W",
        classification=Classification.worker,
    )

    assert manifest.to_dict()["icon"] == "W"


def test_diagram_serializes_included_manifests() -> None:
    """Diagram captures a high-level list of manifests to render together."""
    web = Manifest(name="Web")
    api = Manifest(name="API")
    diagram = Diagram(name="Product", description="Product surface", manifests=[web, api])

    data = diagram.to_dict()

    assert data["name"] == "Product"
    assert data["description"] == "Product surface"
    assert [manifest["name"] for manifest in data["manifests"]] == ["Web", "API"]


def test_diagram_query_resolves_against_manifest_pool() -> None:
    """Diagram can use a query instead of an explicit manifest list."""
    web = Manifest(name="Web")
    api = Manifest(name="API")
    diagram = Diagram(name="Python Services", query=Diagram.Query(name="API"))

    data = diagram.to_dict(manifest_pool=[web, api])

    assert [manifest["name"] for manifest in data["manifests"]] == ["API"]


def test_diagram_rejects_mixed_manifest_selection_modes() -> None:
    """Diagram treats query as an alternative to explicit manifests."""
    api = Manifest(name="API")

    with pytest.raises(ValueError, match="either manifests or query"):
        Diagram(name="Mixed", manifests=[api], query=ManifestQuery(name="API"))
