"""Info enum list contract and removed-field regressions."""

import pytest
from pydantic import ValidationError

from scryr import Info, Manifest, ManifestQuery, MetricTool
from scryr.types import DeploymentTarget, ProgrammingLanguage, WebFramework


def test_info_lists_default_to_empty_without_assuming_a_stack() -> None:
    """Empty lists represent unspecified technology choices."""
    info = Info()
    assert info.languages == info.frameworks == info.deployments == info.metric_tools == []
    info.languages.append(ProgrammingLanguage.python)
    assert Info().languages == []


def test_info_lists_round_trip_typed_members_and_wire_identifiers() -> None:
    """Multiple languages, targets, IaC tools and telemetry tools survive serialization."""
    info = Info(
        languages=[ProgrammingLanguage.python, ProgrammingLanguage.rust],
        frameworks=[WebFramework.fastapi],
        deployments=[DeploymentTarget.cloud_run, DeploymentTarget.terraform],
        metric_tools=[MetricTool.datadog, MetricTool.jaeger, MetricTool.datadog],
    )
    payload = info.model_dump(mode="json")
    assert payload["languages"] == ["python", "rust"]
    assert payload["deployments"] == ["cloud_run", "terraform"]
    assert payload["metricTools"] == ["datadog", "jaeger"]
    assert Info.model_validate(payload) == info
    assert Info.model_validate_json(info.model_dump_json()) == info


@pytest.mark.parametrize("field", ["languages", "frameworks", "deployments", "metric_tools"])
def test_info_rejects_scalar_enum_fields(field: str) -> None:
    """Even valid individual enum values must be wrapped in a list."""
    choices = {
        "languages": "python",
        "frameworks": "fastapi",
        "deployments": "terraform",
        "metric_tools": "datadog",
    }
    with pytest.raises(ValidationError, match=field):
        Info.model_validate({field: choices[field]})


@pytest.mark.parametrize("field", ["languages", "frameworks", "deployments", "metricTools"])
def test_info_rejects_unknown_enum_members(field: str) -> None:
    """List members remain a closed enum vocabulary rather than arbitrary strings."""
    with pytest.raises(ValidationError, match=field):
        Info.model_validate({field: ["unsupported_tool"]})


@pytest.mark.parametrize(
    "field",
    [
        "language",
        "deployment",
        "auth_type",
        "authType",
        "monitoring",
        "tracing",
        "log_aggregation",
        "logAggregation",
        "iac_tool",
        "iacTool",
        "owner_team",
        "ownerTeam",
    ],
)
def test_removed_fields_are_not_accepted_or_exposed(field: str) -> None:
    """Old fields are neither silent aliases nor serialized alongside the new fields."""
    assert field not in Info.model_fields
    assert field not in Info().model_dump()
    with pytest.raises(ValidationError, match="Extra inputs"):
        Info.model_validate({field: "none"})
    with pytest.raises(ValidationError, match="Extra inputs"):
        Manifest.model_validate({field: "none"})
    with pytest.raises(ValueError, match="Unknown or ambiguous Manifest query field"):
        ManifestQuery(where={field: "none"})


def test_owners_is_plain_text_and_serializes_without_an_alias() -> None:
    """Ownership is currently a single text field, rather than an enum or label wrapper."""
    info = Info(owners="Platform and Data Engineering")
    assert type(info.owners) is str
    assert info.model_dump()["owners"] == "Platform and Data Engineering"
    assert Info.model_fields["owners"].annotation is str
    assert Manifest(owners="Platform").info.owners == "Platform"
    with pytest.raises(ValidationError, match="owners"):
        Info.model_validate({"owners": ["Platform"]})


def test_plural_query_fields_match_membership_and_iac_deployments() -> None:
    """Queries use plural technology names and match requested list members."""
    manifest = Manifest(
        languages=[ProgrammingLanguage.python, ProgrammingLanguage.rust],
        deployments=[DeploymentTarget.ec2, DeploymentTarget.terraform],
        metric_tools=[MetricTool.prometheus, MetricTool.loki],
    )
    query = ManifestQuery(
        languages=ProgrammingLanguage.rust,
        deployments=DeploymentTarget.terraform,
        metric_tools=MetricTool.loki,
    )
    assert query.filter([manifest]) == [manifest]
    assert ManifestQuery(metricTools=MetricTool.datadog).filter([manifest]) == []


def test_deployment_identifiers_are_unique_and_all_round_trip() -> None:
    """Provider-specific targets and IaC tools retain unambiguous serialized identifiers."""
    targets = list(DeploymentTarget)
    assert len({target.id for target in targets}) == len(targets)
    info = Info(deployments=targets)
    assert Info.model_validate_json(info.model_dump_json()).deployments == targets
