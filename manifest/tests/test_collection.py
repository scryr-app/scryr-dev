"""Portable authoring and execution-plan boundary regressions."""

from datetime import timedelta
from pathlib import Path
from types import ModuleType
from typing import Annotated, Any, cast

import pytest
from pydantic import (
    AfterValidator,
    AliasChoices,
    BaseModel,
    ConfigDict,
    Field,
    ValidationError,
    WithJsonSchema,
    field_validator,
)

from scryr import Collector, Command, Every, JsonOutput, Manifest, StatusCard, Task
from scryr.collection import collection_plan, portable_schema
from scryr.runtime import emit_scryr_values, load_manifest_module


class Snapshot(BaseModel):
    """A small portable output model."""

    healthy: bool
    count: int = Field(ge=0)


def task(identifier: str = "check") -> Task[Snapshot]:
    """Construct a test-only execution declaration."""
    return Task[Snapshot](
        id=identifier, command=Command(argv=["printf", "{}"]), output=JsonOutput(model=Snapshot)
    )


def test_sample_emits_one_plan_with_shared_collector() -> None:
    """Multiple cards share one scheduled execution and preserve stable target IDs."""
    module = load_manifest_module(Path(__file__).parent / "samples/collectors/index.scry")
    plans = [
        record["collection_plan"]
        for record in emit_scryr_values(module)
        if record["kind"] == "collection_plan"
    ]
    assert any(record["kind"] == "diagram" for record in emit_scryr_values(module))
    assert len(plans) == 1
    plan = cast("dict[str, Any]", plans[0])
    assert isinstance(plan, dict)
    assert len(plan["tasks"]) == len(plan["collectors"]) == 1
    assert len(plan["cards"]) == 3
    assert {card["collector_id"] for card in plan["cards"]} == {"api-deployment"}
    assert {card["target"] for card in plan["cards"]} == {"collectors-api"}
    assert plan["tasks"][0]["output_schema"]["properties"]["ready_instances"]["minimum"] == 0


def test_discovery_follows_private_dependencies() -> None:
    """A public card is enough to discover referenced tasks and collectors."""
    module = ModuleType("example")
    source = Collector[Snapshot](id="collect", task=task(), schedule=Every(timedelta(seconds=10)))
    module.__dict__["card"] = StatusCard[Snapshot](
        id="card", title="Health", source=source, target=Manifest(name="API", manifest_id="api")
    )
    module.__dict__["api"] = module.__dict__["card"].target
    plan = collection_plan(module)
    assert plan is not None
    assert plan["tasks"][0]["id"] == "check"
    assert plan["collectors"][0]["interval_seconds"] == 10


def test_absent_plan_preserves_existing_records() -> None:
    """Architecture-only modules gain no execution metadata."""
    assert collection_plan(ModuleType("empty")) is None


def test_conflicting_ids_rejected() -> None:
    """Two incompatible definitions cannot silently overwrite each other."""
    module = ModuleType("conflicts")
    module.__dict__["first"] = task()
    module.__dict__["second"] = task().model_copy(update={"timeout": timedelta(seconds=4)})
    with pytest.raises(ValueError, match="Conflicting"):
        collection_plan(module)


def test_target_requires_stable_identity() -> None:
    """Cards cannot use generated or mutable names as target references."""
    source = Collector[Snapshot](id="collect", task=task(), schedule=Every(timedelta(seconds=10)))
    with pytest.raises(ValidationError, match="manifest_id"):
        StatusCard(id="card", title="Health", source=source, target=Manifest(name="API"))


@pytest.mark.parametrize(
    "duration", [timedelta(0), timedelta(seconds=-1), timedelta(milliseconds=500)]
)
def test_duration_requires_positive_whole_seconds(duration: timedelta) -> None:
    """Reject intervals the Rust runner cannot represent exactly."""
    with pytest.raises(ValueError, match="whole seconds"):
        Every(duration)


def test_output_rejects_python_validator() -> None:
    """JSON Schema export must not silently drop Python business rules."""

    class Validated(BaseModel):
        count: int

        @field_validator("count")
        @classmethod
        def positive(cls, value: int) -> int:
            return abs(value)

    with pytest.raises(ValueError, match="Python validators"):
        JsonOutput(model=Validated)


def test_output_rejects_annotated_validator() -> None:
    """Annotated callbacks also cannot cross the Rust contract boundary."""

    class Validated(BaseModel):
        count: Annotated[int, AfterValidator(abs)]

    with pytest.raises(ValueError, match="Python validators"):
        portable_schema(Validated)


def test_output_rejects_unsupported_schema() -> None:
    """Reject constraints the Rust validator cannot enforce."""

    class Pattern(BaseModel):
        name: str = Field(pattern="^a")

    with pytest.raises(ValueError, match="pattern"):
        portable_schema(Pattern)


def test_output_rejects_recursive_schema() -> None:
    """Bound validation complexity by rejecting recursive output types."""

    class Recursive(BaseModel):
        children: list[Recursive]

    with pytest.raises(ValueError, match="nonrecursive"):
        portable_schema(Recursive)


def test_mismatched_output_type_rejected() -> None:
    """The generic task contract rejects an unrelated output model."""

    class Other(BaseModel):
        version: str

    with pytest.raises(ValidationError):
        Task[Snapshot].model_validate(
            {
                "id": "mismatch",
                "command": {"argv": ["printf", "{}"]},
                "output": {"model": Other},
            }
        )


def test_mismatched_card_source_type_rejected() -> None:
    """Typed cards cannot silently bind to a collector of another report type."""

    class Other(BaseModel):
        version: str

    source = Collector[Snapshot](id="collect", task=task(), schedule=Every(timedelta(seconds=10)))
    with pytest.raises(ValidationError):
        StatusCard[Other].model_validate(
            {
                "id": "card",
                "title": "Other",
                "source": source,
                "target": Manifest(name="API", manifest_id="api"),
            }
        )


def test_output_rejects_config_constraints_missing_from_schema() -> None:
    """Model string constraints must not disappear when exported to Rust."""

    class Configured(BaseModel):
        model_config = ConfigDict(str_min_length=3)
        name: str

    with pytest.raises(ValueError, match="string config"):
        portable_schema(Configured)


def test_output_rejects_post_init_validation() -> None:
    """Initialization hooks are Python behavior, even without decorators."""

    class Configured(BaseModel):
        name: str

        def model_post_init(self, context: object) -> None:  # noqa: ARG002 - Pydantic override.
            """Example of a business constraint not represented in JSON Schema."""
            if not self.name:
                msg = "Name is required"
                raise ValueError(msg)

    with pytest.raises(ValueError, match="initialization hooks"):
        portable_schema(Configured)


def test_output_rejects_schema_override() -> None:
    """Custom schema annotations cannot hide a field's actual validation."""

    class Configured(BaseModel):
        count: Annotated[int, Field(ge=1), WithJsonSchema({"type": "integer"})]

    with pytest.raises(ValueError, match="customize JSON Schema"):
        portable_schema(Configured)


def test_output_rejects_alias_choices() -> None:
    """Multiple accepted aliases do not have a portable exported contract."""

    class Configured(BaseModel):
        count: int = Field(validation_alias=AliasChoices("count", "total"))

    with pytest.raises(ValueError, match="single string validation alias"):
        portable_schema(Configured)


def test_output_preserves_simple_alias() -> None:
    """A simple alias is the exact key expected from command JSON."""

    class Configured(BaseModel):
        count: int = Field(alias="total")

    schema = portable_schema(Configured)
    assert schema["required"] == ["total"]
    assert "total" in schema["properties"]


def test_export_revalidates_mutated_task() -> None:
    """In-place list mutations cannot bypass declaration validation."""
    module = ModuleType("mutated")
    operation = task()
    operation.command.argv.clear()
    module.__dict__["operation"] = operation
    with pytest.raises(ValidationError):
        collection_plan(module)


def test_card_requires_emitted_target() -> None:
    """Cards cannot bind to architecture objects absent from the exported map."""
    module = ModuleType("missing_target")
    source = Collector[Snapshot](id="collect", task=task(), schedule=Every(timedelta(seconds=10)))
    module.__dict__["card"] = StatusCard[Snapshot](
        id="card", title="Health", source=source, target=Manifest(name="API", manifest_id="api")
    )
    with pytest.raises(ValueError, match="public Manifest"):
        collection_plan(module)
    module.__dict__["api"] = module.__dict__["card"].target
    module.__dict__["duplicate"] = Manifest(name="Other", manifest_id="api")
    with pytest.raises(ValueError, match="Duplicate manifest_id"):
        collection_plan(module)


def test_output_rejects_out_of_range_numeric_bounds() -> None:
    """Schema bounds cannot be rounded when parsed by Rust's JSON number type."""

    class Configured(BaseModel):
        count: int = Field(ge=2**64)

    with pytest.raises(ValueError, match="64-bit"):
        portable_schema(Configured)
