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
    GithubManifestSection,
    InfoManifestSection,
    Link,
    Manifest,
    ManifestQuery,
)
from scryr.types import (
    AuthType,
    CalendarVersion,
    Classification,
    DeploymentTarget,
    IaCToolType,
    LogAggregationType,
    MonitoringType,
    ProgrammingLanguage,
    SemVer,
    TracingType,
    Url,
    WebFramework,
)


def test_manifest_dict_serializes_enums_and_version_to_strings() -> None:
    """Manifest.dict converts enums and version models into JSON-friendly identifiers."""
    manifest = Manifest(
        name="SerializeMe",
        classification=Classification.internal_api,
        info=InfoManifestSection(
            version=SemVer("1.2.3"),
            language=ProgrammingLanguage.javascript,
            frameworks=[WebFramework.express, WebFramework.nextjs],
            deployment=DeploymentTarget.cloud_run,
            auth_type=AuthType.oauth2,
            monitoring=MonitoringType.datadog,
            log_aggregation=LogAggregationType.elk_stack,
            tracing=TracingType.jaeger,
            iac_tool=IaCToolType.terraform,
        ),
    )

    data = manifest.dict()

    info = data["info"]
    assert info["version"] == "1.2.3"
    assert info["language"] == "javascript"
    assert info["frameworks"] == ["express", "nextjs"]
    assert info["deployment"] == "cloud_run"
    assert data["classification"] == "internal_api"
    assert info["authType"] == "oauth2"
    assert info["monitoring"] == "datadog"
    assert info["logAggregation"] == "elk_stack"
    assert info["tracing"] == "jaeger"
    assert info["iacTool"] == "terraform"
    assert data["icon"] == ""


def test_manifest_to_dict_returns_plain_json_safe_types() -> None:
    """to_dict returns JSON-safe primitives and lists, including version as a string."""
    manifest = Manifest(
        name="JsonSafe",
        info=InfoManifestSection(
            version=CalendarVersion("2026.3"),
            links=[Link(site_name="Docs", http_url=Url("https://example.com/docs"))],
            docs=[Url("https://docs.example.com"), "README"],
        ),
        github=GithubManifestSection(repo_url=Url("https://example.com/repo")),
    )

    data = manifest.to_dict()

    assert isinstance(data, dict)
    assert data["info"]["version"] == "2026.3"
    assert data["github"]["repoUrl"] == "https://example.com/repo"
    assert data["info"]["links"][0]["site_name"] == "Docs"
    assert data["info"]["links"][0]["http_url"] == "https://example.com/docs"
    assert data["info"]["docs"] == ["https://docs.example.com", "README"]


def test_manifest_exposes_embedded_section_constructors() -> None:
    """Manifest exposes short constructors for embedding typed sections."""
    manifest = Manifest(
        name="EmbeddedSections",
        info=Manifest.Info(description="visible"),
        github=Manifest.Github(repo_url=Url("https://example.com/repo")),
    )

    assert isinstance(manifest.info, InfoManifestSection)
    assert isinstance(manifest.github, GithubManifestSection)
    assert manifest.to_dict()["info"]["description"] == "visible"
    assert manifest.to_dict()["github"]["repoUrl"] == "https://example.com/repo"


def test_manifest_accepts_flat_section_constructor_fields() -> None:
    """Manifest accepts common section fields directly while serializing sections."""
    manifest = Manifest(
        name="FlatSections",
        description="visible",
        language=ProgrammingLanguage.javascript,
        frameworks=[WebFramework.nextjs],
        repo_url=Url("https://example.com/repo"),
        cicd_tool="github_actions",
    )

    data = manifest.to_dict()

    assert data["info"]["description"] == "visible"
    assert data["info"]["language"] == "javascript"
    assert data["info"]["frameworks"] == ["nextjs"]
    assert data["github"]["repoUrl"] == "https://example.com/repo"
    assert data["cicd"]["platform"] == "github_actions"


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
    manifest = Manifest(name="IncludeOnly", info=InfoManifestSection(description="visible"))

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
    web = Manifest(name="Web", language=ProgrammingLanguage.typescript)
    api = Manifest(name="API", language=ProgrammingLanguage.python)
    diagram = Diagram(name="Product", description="Product surface", manifests=[web, api])

    data = diagram.to_dict()

    assert data["name"] == "Product"
    assert data["description"] == "Product surface"
    assert [manifest["name"] for manifest in data["manifests"]] == ["Web", "API"]


def test_manifest_query_filters_by_shorthand_and_list_fields() -> None:
    """ManifestQuery matches common Pydantic fields with compact keyword syntax."""
    api = Manifest(
        name="API",
        language=ProgrammingLanguage.python,
        frameworks=[WebFramework.fastapi],
        tags=["backend", "public"],
    )
    web = Manifest(
        name="Web",
        language=ProgrammingLanguage.typescript,
        frameworks=[WebFramework.nextjs],
        tags=["frontend", "public"],
    )

    query = ManifestQuery(
        language=ProgrammingLanguage.python,
        frameworks=WebFramework.fastapi,
        tags="backend",
    )

    assert query.filter([api, web]) == [api]
    assert query.model_dump(mode="json", by_alias=True)["where"] == {
        "info.language": "python",
        "info.frameworks": "fastapi",
        "tags": "backend",
    }


def test_manifest_query_filters_by_nested_field_paths() -> None:
    """ManifestQuery accepts nested mappings and Python-safe double-underscore paths."""
    api = Manifest(
        name="API",
        github=GithubManifestSection(repo_url=Url("https://example.com/api")),
    )
    web = Manifest(
        name="Web",
        github=GithubManifestSection(repo_url=Url("https://example.com/web")),
    )

    query = ManifestQuery(
        where={"github": {"repoUrl": "https://example.com/api"}},
        info__language=ProgrammingLanguage.python,
    )

    assert query.filter([api, web]) == [api]


def test_diagram_query_resolves_against_manifest_pool() -> None:
    """Diagram can use a query instead of an explicit manifest list."""
    web = Manifest(name="Web", language=ProgrammingLanguage.typescript)
    api = Manifest(name="API", language=ProgrammingLanguage.python)
    diagram = Diagram(name="Python Services", query=Diagram.Query(language="python"))

    data = diagram.to_dict(manifest_pool=[web, api])

    assert [manifest["name"] for manifest in data["manifests"]] == ["API"]


def test_diagram_rejects_mixed_manifest_selection_modes() -> None:
    """Diagram treats query as an alternative to explicit manifests."""
    api = Manifest(name="API", language=ProgrammingLanguage.python)

    with pytest.raises(ValueError, match="either manifests or query"):
        Diagram(name="Mixed", manifests=[api], query=ManifestQuery(language="python"))
