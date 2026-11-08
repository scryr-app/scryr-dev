"""GitHub delivery source configuration and serialization contracts."""

import json
from datetime import UTC, datetime

import pytest
from pydantic import ValidationError

from scryr import Manifest
from scryr.delivery import (
    ActionsBuildSource,
    ActionsReportSource,
    DeliveryEnvironment,
    DeliveryReportSchema,
    DeliveryRuntimeEvidence,
    DeliveryRuntimeWorkload,
    DeliveryTask,
    DeliveryTasks,
    DeliveryVerificationTask,
    GitHubDelivery,
    GitHubDeploymentSource,
    GitHubRepository,
)


def delivery() -> GitHubDelivery:
    """A source definition contains selectors, never live data or credentials."""
    return GitHubDelivery(
        repository=GitHubRepository(owner="example", name="payments"),
        builds=ActionsBuildSource(
            workflow=".github/workflows/build.yml", artifact_names=["payments"]
        ),
        environments=[
            DeliveryEnvironment(
                name="production",
                deployments=GitHubDeploymentSource(environment="production"),
                runtime=ActionsReportSource(
                    workflow=".github/workflows/runtime.yml",
                    artifact_name="runtime",
                    report_schema=DeliveryReportSchema.runtime,
                ),
            ),
        ],
    )


def test_delivery_roundtrip_and_legacy_default() -> None:
    """Wire aliases survive Python JSON reconstruction without executing tools."""
    manifest = Manifest(name="Payments", manifest_id="payments", delivery=delivery())
    encoded = manifest.model_dump_json()
    wire = json.loads(encoded)
    assert wire["delivery"]["builds"]["artifactNames"] == ["payments"]
    assert wire["delivery"]["environments"][0]["runtime"]["schema"] == "scryr.runtime.v1"
    assert Manifest.model_validate_json(encoded).delivery == manifest.delivery
    assert Manifest(name="Legacy").delivery is None


def test_delivery_requires_stable_identity() -> None:
    """Operational evidence must not be attached by mutable block display name."""
    with pytest.raises(ValidationError, match="stable manifest_id"):
        Manifest(name="Payments", delivery=delivery())


def test_delivery_owns_typed_tasks_and_legacy_defaults() -> None:
    """Task settings survive serialization and old definitions receive useful defaults."""
    source = delivery()
    source.tasks = DeliveryTasks(
        collect=DeliveryTask(limit=5), verify=DeliveryVerificationTask(limit=10)
    )
    wire = json.loads(source.model_dump_json())
    assert wire["tasks"] == {
        "collect": {"limit": 5, "verifyAttestations": False},
        "verify": {"limit": 10, "verifyAttestations": True},
    }
    assert GitHubDelivery.model_validate(wire).tasks == source.tasks
    del wire["tasks"]
    assert GitHubDelivery.model_validate(wire).tasks == DeliveryTasks()
    assert DeliveryTasks.model_validate({"verify": {}}).verify.verify_attestations is True
    for limit in [0, 101]:
        with pytest.raises(ValidationError):
            DeliveryTask(limit=limit)
    with pytest.raises(ValidationError, match="Extra inputs"):
        DeliveryTasks.model_validate({"deploy": {"run": "echo nope"}})


def test_report_schema_enum_preserves_wire_values() -> None:
    """Authors use enum members while existing serialized schema strings still load."""
    source = ActionsReportSource.model_validate(
        {
            "workflow": ".github/workflows/runtime.yml",
            "artifactName": "runtime",
            "schema": "scryr.runtime.v1",
        }
    )
    assert source.report_schema is DeliveryReportSchema.runtime
    assert json.loads(source.model_dump_json())["schema"] == "scryr.runtime.v1"


@pytest.mark.parametrize("owner", ["", "../owner", "owner/repo", "--hostname=x", "a b"])
def test_invalid_repository_selectors(owner: str) -> None:
    """Repository selectors cannot introduce CLI options or URL path segments."""
    with pytest.raises(ValidationError):
        GitHubRepository(owner=owner, name="payments")


def test_invalid_evidence_bindings() -> None:
    """Reject duplicate environments, wrong schemas and mismatched deployments."""
    source = delivery()
    with pytest.raises(ValidationError, match="unique"):
        GitHubDelivery(
            repository=source.repository,
            builds=source.builds,
            environments=source.environments * 2,
        )
    with pytest.raises(ValidationError, match="must match"):
        DeliveryEnvironment(
            name="production", deployments=GitHubDeploymentSource(environment="staging")
        )
    with pytest.raises(ValidationError, match=r"scryr\.runtime\.v1"):
        DeliveryEnvironment(
            name="production",
            deployments=GitHubDeploymentSource(environment="production"),
            runtime=ActionsReportSource(
                workflow=".github/workflows/runtime.yml",
                artifact_name="runtime",
                report_schema=DeliveryReportSchema.compatibility,
            ),
        )


def test_unknown_fields_and_credentials_are_rejected() -> None:
    """A definition cannot silently accept misspellings or embedded access tokens."""
    wire = delivery().model_dump()
    wire["token"] = "not-a-secret"  # noqa: S105 - deliberate rejected fixture
    with pytest.raises(ValidationError, match="Extra inputs"):
        GitHubDelivery.model_validate(wire)


def test_runtime_report_is_typed_and_versioned() -> None:
    """Workflows can emit validated camelCase reports without hand-writing dictionaries."""
    workload = DeliveryRuntimeWorkload(
        name="api", artifact_digest="sha256:" + "a" * 64, replicas=3, ready_replicas=2
    )
    report = DeliveryRuntimeEvidence(
        manifest_id="payments",
        environment="production",
        deployment_id="123",
        configuration_revision="config-1",
        schema_revision="schema-2",
        observed_at=datetime.now(UTC),
        workloads=[workload],
    )
    wire = json.loads(report.model_dump_json())
    assert wire["schema"] == "scryr.runtime.v1"
    assert wire["workloads"][0]["readyReplicas"] == 2
    assert DeliveryRuntimeEvidence.model_validate_json(report.model_dump_json()) == report
    with pytest.raises(ValidationError, match="cannot exceed"):
        DeliveryRuntimeWorkload(
            name="api", artifact_digest="sha256:" + "a" * 64, replicas=1, ready_replicas=2
        )
    wire["observedAt"] = "2026-11-04T12:00:00"
    with pytest.raises(ValidationError, match="timezone"):
        DeliveryRuntimeEvidence.model_validate(wire)
