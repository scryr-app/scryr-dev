"""Tests for Manifest helper methods and string representation."""

from __future__ import annotations

from scryr.manifest import Info, Manifest
from scryr.types import DeploymentTarget, ProgrammingLanguage, WebFramework


def test_framework_names_returns_framework_identifiers() -> None:
    """framework_names returns the string ids of selected frameworks."""
    manifest = Manifest(
        name="FrameworkList",
        info=Info(frameworks=[WebFramework.fastapi, WebFramework.nextjs]),
    )

    assert manifest.framework_names() == ["fastapi", "nextjs"]


def test_manifest_str_representation_uses_language_and_deployment() -> None:
    """__str__ includes name, language, and deployment target id."""
    manifest = Manifest(
        name="Stringified",
        info=Info(
            language=ProgrammingLanguage.rust,
            deployment=DeploymentTarget.k8s_cluster,
        ),
    )

    assert str(manifest) == "<Stringified (rust) -> k8s_cluster>"
