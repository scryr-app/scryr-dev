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
    Info,
    Link,
    Manifest,
    ManifestQuery,
)
from scryr.types import (
    CalendarVersion,
    Classification,
    DeploymentTarget,
    MetricTool,
    ProgrammingLanguage,
    SemVer,
    Url,
    WebFramework,
)


def test_manifest_dict_serializes_enums_and_version_to_strings() -> None:
    """Manifest.dict converts enums and version models into JSON-friendly identifiers."""
    manifest = Manifest(
        name="SerializeMe",
        classification=Classification.internal_api,
        info=Info(
            version=SemVer("1.2.3"),
            languages=[ProgrammingLanguage.javascript],
            frameworks=[WebFramework.express, WebFramework.nextjs],
            deployments=[DeploymentTarget.cloud_run, DeploymentTarget.terraform],
            metric_tools=[MetricTool.datadog, MetricTool.elk_stack, MetricTool.jaeger],
        ),
    )

    data = manifest.dict()

    info = data["info"]
    assert info["version"] == "1.2.3"
    assert info["languages"] == ["javascript"]
    assert info["frameworks"] == ["express", "nextjs"]
    assert info["deployments"] == ["cloud_run", "terraform"]
    assert data["classification"] == "internal_api"
    assert info["metricTools"] == ["datadog", "elk_stack", "jaeger"]
    assert data["icon"] == ""


def test_manifest_to_dict_returns_plain_json_safe_types() -> None:
    """to_dict returns JSON-safe primitives and lists, including version as a string."""
    manifest = Manifest(
        name="JsonSafe",
        info=Info(
            version=CalendarVersion("2026.3"),
            links=[Link(site_name="Docs", http_url=Url("https://example.com/docs"))],
            docs=[Url("https://docs.example.com"), "README"],
        ),
    )

    data = manifest.to_dict()

    assert isinstance(data, dict)
    assert data["info"]["version"] == "2026.3"
    assert data["info"]["links"][0]["site_name"] == "Docs"
    assert data["info"]["links"][0]["http_url"] == "https://example.com/docs"
    assert data["info"]["docs"] == ["https://docs.example.com", "README"]


def test_manifest_exposes_embedded_section_constructors() -> None:
    """Manifest exposes short constructors for embedding typed sections."""
    manifest = Manifest(
        name="EmbeddedSections",
        info=Manifest.Info(description="visible"),
    )

    assert isinstance(manifest.info, Info)
    assert manifest.to_dict()["info"]["description"] == "visible"


def test_manifest_accepts_flat_section_constructor_fields() -> None:
    """Manifest accepts common section fields directly while serializing sections."""
    manifest = Manifest(
        name="FlatSections",
        description="visible",
        languages=[ProgrammingLanguage.javascript],
        frameworks=[WebFramework.nextjs],
    )

    data = manifest.to_dict()

    assert data["info"]["description"] == "visible"
    assert data["info"]["languages"] == ["javascript"]
    assert data["info"]["frameworks"] == ["nextjs"]


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
    manifest = Manifest(name="IncludeOnly", info=Info(description="visible"))

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
    web = Manifest(name="Web", languages=[ProgrammingLanguage.typescript])
    api = Manifest(name="API", languages=[ProgrammingLanguage.python])
    diagram = Diagram(name="Product", description="Product surface", manifests=[web, api])

    data = diagram.to_dict()

    assert data["name"] == "Product"
    assert data["description"] == "Product surface"
    assert [manifest["name"] for manifest in data["manifests"]] == ["Web", "API"]


def test_manifest_query_filters_by_shorthand_and_list_fields() -> None:
    """ManifestQuery matches common Pydantic fields with compact keyword syntax."""
    api = Manifest(
        name="API",
        languages=[ProgrammingLanguage.python],
        frameworks=[WebFramework.fastapi],
        tags=["backend", "public"],
    )
    web = Manifest(
        name="Web",
        languages=[ProgrammingLanguage.typescript],
        frameworks=[WebFramework.nextjs],
        tags=["frontend", "public"],
    )

    query = ManifestQuery(
        languages=ProgrammingLanguage.python,
        frameworks=WebFramework.fastapi,
        tags="backend",
    )

    assert query.filter([api, web]) == [api]
    assert query.model_dump(mode="json", by_alias=True)["where"] == {
        "info.languages": "python",
        "info.frameworks": "fastapi",
        "tags": "backend",
    }


def test_manifest_query_filters_by_nested_field_paths() -> None:
    """ManifestQuery accepts nested mappings and Python-safe double-underscore paths."""
    api = Manifest(
        name="API",
        info=Info(owners="backend", languages=[ProgrammingLanguage.python]),
    )
    web = Manifest(
        name="Web",
        info=Info(owners="frontend"),
    )

    query = ManifestQuery(
        where={"info": {"owners": "backend"}},
        info__languages=ProgrammingLanguage.python,
    )

    assert query.filter([api, web]) == [api]


def test_diagram_query_resolves_against_manifest_pool() -> None:
    """Diagram can use a query instead of an explicit manifest list."""
    web = Manifest(name="Web", languages=[ProgrammingLanguage.typescript])
    api = Manifest(name="API", languages=[ProgrammingLanguage.python])
    diagram = Diagram(name="Python Services", query=Diagram.Query(languages="python"))

    data = diagram.to_dict(manifest_pool=[web, api])

    assert [manifest["name"] for manifest in data["manifests"]] == ["API"]


def test_diagram_rejects_mixed_manifest_selection_modes() -> None:
    """Diagram treats query as an alternative to explicit manifests."""
    api = Manifest(name="API", languages=[ProgrammingLanguage.python])

    with pytest.raises(ValueError, match="either manifests or query"):
        Diagram(name="Mixed", manifests=[api], query=ManifestQuery(languages="python"))


@pytest.mark.parametrize("classification", list(Classification))
def test_classification_round_trips_roles_and_interfaces(classification: Classification) -> None:
    """Every role and interface remains a scalar classification in persisted JSON."""
    manifest = Manifest(name="Component", classification=classification)
    data = manifest.to_dict()

    assert data["classification"] == classification.value
    assert Manifest.model_validate(data).classification is classification


def test_interface_type_imports_alias_classification() -> None:
    """Legacy imports construct members of the unified classification enum."""
    from scryr import InterfaceType as RootInterfaceType
    from scryr.saas import InterfaceType as SaasInterfaceType
    from scryr.types import InterfaceType

    assert RootInterfaceType is SaasInterfaceType is InterfaceType is Classification
    assert (
        Manifest(name="API", classification=InterfaceType.rest).to_dict()["classification"]
        == "rest"
    )


def test_classification_preserves_defaults_and_legacy_identifiers() -> None:
    """Adding interfaces does not change stored roles or the default classification."""
    assert Manifest(name="Default").classification is Classification.public_api
    assert Classification("network_router") is Classification.network_gateway
    assert Classification.rest.value == "rest"
    assert Classification.graphql.value == "graphql"
    assert Classification.sql.value == "sql"
