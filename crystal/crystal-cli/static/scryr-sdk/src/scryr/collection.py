"""Typed declarations for local Rust-executed collectors and their cards."""

from __future__ import annotations

import math
from datetime import timedelta
from typing import TYPE_CHECKING, Any, Literal, Self

from pydantic import BaseModel, ConfigDict, Field, model_validator

from .manifest import Manifest

if TYPE_CHECKING:
    from types import ModuleType

_SCHEMA_KEYS = {
    "$defs",
    "$ref",
    "title",
    "description",
    "default",
    "type",
    "properties",
    "required",
    "additionalProperties",
    "items",
    "enum",
    "const",
    "anyOf",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "minLength",
    "maxLength",
    "minItems",
    "maxItems",
}


def _seconds(value: timedelta, maximum: int = 2_147_483_647) -> int:
    seconds = value.total_seconds()
    if not 0 < seconds <= maximum or not seconds.is_integer():
        msg = f"Duration must be whole seconds between 1 and {maximum}"
        raise ValueError(msg)
    return int(seconds)


def _portable_core(node: object) -> None:
    if isinstance(node, dict):
        if (
            str(node.get("type", "")).startswith("function-")
            or node.get("computed_fields")
            or node.get("post_init")
            or node.get("custom_init")
        ):
            msg = (
                "Collector output models cannot use Python validators, serializers, "
                "computed fields, or initialization hooks"
            )
            raise ValueError(msg)
        if any(node.get(key) for key in ("strip_whitespace", "to_lower", "to_upper")):
            msg = "Collector output string transformations are not portable"
            raise ValueError(msg)
        model = node.get("cls")
        if isinstance(model, type) and issubclass(model, BaseModel):
            unsupported_config = {
                "str_min_length",
                "str_max_length",
                "str_strip_whitespace",
                "str_to_lower",
                "str_to_upper",
                "json_schema_extra",
                "json_schema_mode_override",
            } & model.model_config.keys()
            custom_schema = any(
                getattr(getattr(model, name), "__func__", None)
                is not getattr(getattr(BaseModel, name), "__func__", None)
                for name in ("__get_pydantic_core_schema__", "__get_pydantic_json_schema__")
            )
            if unsupported_config or custom_schema:
                msg = "Collector output models cannot customize schema generation or string config"
                raise ValueError(msg)
        metadata = node.get("metadata", {})
        if isinstance(metadata, dict) and (
            metadata.get("pydantic_js_annotation_functions") or metadata.get("pydantic_js_extra")
        ):
            msg = "Collector output fields cannot customize JSON Schema"
            raise ValueError(msg)
        alias = node.get("validation_alias")
        if alias is not None and not isinstance(alias, str):
            msg = "Collector output fields support only a single string validation alias"
            raise ValueError(msg)
        for value in node.values():
            _portable_core(value)
    elif isinstance(node, list):
        for value in node:
            _portable_core(value)


def _portable_numbers(node: object) -> None:
    if isinstance(node, int) and not -(2**63) <= node <= 2**64 - 1:
        msg = "Collector schema integers must fit signed or unsigned 64-bit values"
        raise ValueError(msg)
    if isinstance(node, float) and not math.isfinite(node):
        msg = "Collector schema numbers must be finite"
        raise ValueError(msg)
    if isinstance(node, dict):
        for value in node.values():
            _portable_numbers(value)
    elif isinstance(node, list):
        for value in node:
            _portable_numbers(value)


def portable_schema(model: type[BaseModel]) -> dict[str, Any]:
    """Export a bounded JSON Schema subset supported by the Rust runner."""
    _portable_core(model.__pydantic_core_schema__)
    schema = model.model_json_schema()
    _portable_numbers(schema)

    def visit(node: object, refs: frozenset[str] = frozenset(), depth: int = 0) -> None:
        if depth > 64 or not isinstance(node, dict):
            msg = "Collector output schema must be an object with depth at most 64"
            raise ValueError(msg)
        unsupported = node.keys() - _SCHEMA_KEYS
        if unsupported:
            msg = f"Unsupported collector output schema keywords: {sorted(unsupported)}"
            raise ValueError(msg)
        if "$ref" in node:
            ref = node["$ref"]
            if not isinstance(ref, str) or not ref.startswith("#/$defs/") or ref in refs:
                msg = "Collector output references must be local and nonrecursive"
                raise ValueError(msg)
            name = ref.removeprefix("#/$defs/")
            if not name or "/" in name:
                msg = "Collector output references must name a local definition"
                raise ValueError(msg)
            definition = schema.get("$defs", {}).get(name.replace("~1", "/").replace("~0", "~"))
            visit(definition, refs | {ref}, depth + 1)
        for key in ("$defs", "properties"):
            for child in node.get(key, {}).values():
                visit(child, refs, depth + 1)
        for key in ("items", "additionalProperties"):
            if key in node and not isinstance(node[key], bool):
                visit(node[key], refs, depth + 1)
        for child in node.get("anyOf", []):
            visit(child, refs, depth + 1)

    visit(schema)
    return schema


class _Declaration(BaseModel):
    model_config = ConfigDict(extra="forbid", revalidate_instances="always")


class Command(_Declaration):
    """A noninteractive argument-array command, relative to the manifest root."""

    argv: list[str] = Field(min_length=1)
    cwd: str = "."

    @model_validator(mode="after")
    def validate_command(self) -> Self:
        """Reject empty executables and NUL-containing process arguments."""
        if not self.argv[0] or any("\0" in part for part in [*self.argv, self.cwd]):
            msg = "Commands require a nonempty executable and cannot contain NUL bytes"
            raise ValueError(msg)
        return self


class JsonOutput[T: BaseModel](_Declaration):
    """The portable Pydantic contract for a command's JSON stdout."""

    model: type[T]

    @model_validator(mode="after")
    def validate_output(self) -> Self:
        """Reject Python-only behavior before the execution plan is emitted."""
        portable_schema(self.model)
        return self


class Task[T: BaseModel](_Declaration):
    """A reusable command and its typed output, without a schedule."""

    id: str = Field(min_length=1)
    command: Command
    output: JsonOutput[T]
    timeout: timedelta = timedelta(seconds=30)
    max_output_bytes: int = Field(default=1_048_576, ge=1, le=16_777_216)
    accepted_exit_codes: list[int] = Field(default_factory=lambda: [0], min_length=1)

    @model_validator(mode="after")
    def validate_timeout(self) -> Self:
        """Require a bounded whole-second timeout."""
        _seconds(self.timeout, 86_400)
        return self

    def to_plan(self) -> dict[str, Any]:
        """Serialize the executable contract without a Python class reference."""
        return {
            "id": self.id,
            "command": self.command.model_dump(),
            "output_schema": portable_schema(self.output.model),
            "timeout_seconds": _seconds(self.timeout, 86_400),
            "max_output_bytes": self.max_output_bytes,
            "accepted_exit_codes": self.accepted_exit_codes,
        }


class Every(_Declaration):
    """A fixed interval schedule; missed intervals are skipped."""

    interval: timedelta

    def __init__(self, interval: timedelta) -> None:
        """Accept the interval as a positional or named argument."""
        data = {"interval": interval}
        super().__init__(**data)
        _seconds(self.interval)


class Collector[T: BaseModel](_Declaration):
    """A task scheduled on the local runner, with freshness and retention."""

    id: str = Field(min_length=1)
    task: Task[T]
    schedule: Every
    environment: str = Field(default="local", min_length=1)
    stale_after: timedelta = timedelta(minutes=2)
    retain_for: timedelta = timedelta(days=30)

    @model_validator(mode="after")
    def validate_durations(self) -> Self:
        """Require bounded whole-second freshness and retention durations."""
        _seconds(self.stale_after)
        _seconds(self.retain_for)
        return self

    def to_plan(self) -> dict[str, Any]:
        """Serialize references by stable ID."""
        return {
            "id": self.id,
            "task_id": self.task.id,
            "environment": self.environment,
            "interval_seconds": _seconds(self.schedule.interval),
            "stale_after_seconds": _seconds(self.stale_after),
            "retain_for_seconds": _seconds(self.retain_for),
        }


class _Card[T: BaseModel](_Declaration):
    id: str = Field(min_length=1)
    title: str = Field(min_length=1)
    source: Collector[T]
    target: Manifest
    kind: Literal["status", "table", "history"]

    @model_validator(mode="after")
    def validate_target(self) -> Self:
        """Require an explicit stable architecture identity."""
        if not self.target.manifest_id:
            msg = "Card targets require an explicit manifest_id"
            raise ValueError(msg)
        return self

    def to_plan(self) -> dict[str, Any]:
        """Serialize a read-only card binding."""
        return {
            "id": self.id,
            "kind": self.kind,
            "title": self.title,
            "collector_id": self.source.id,
            "target": self.target.manifest_id,
        }


class StatusCard[T: BaseModel](_Card[T]):
    """Display the latest structured observation and its freshness."""

    kind: Literal["status"] = "status"


class TableCard[T: BaseModel](_Card[T]):
    """Display the latest observation as a property table."""

    kind: Literal["table"] = "table"


class HistoryCard[T: BaseModel](_Card[T]):
    """Display recent collector observations, newest first."""

    kind: Literal["history"] = "history"


def collection_plan(module: ModuleType) -> dict[str, Any] | None:
    """Discover declarations and referenced dependencies once, in stable ID order."""
    tasks: dict[str, dict[str, Any]] = {}
    collectors: dict[str, dict[str, Any]] = {}
    cards: dict[str, dict[str, Any]] = {}
    targets: dict[str, Manifest] = {}
    duplicate_targets: set[str] = set()
    for name, value in vars(module).items():
        if not name.startswith("_") and isinstance(value, Manifest) and value.manifest_id:
            if value.manifest_id in targets and targets[value.manifest_id] is not value:
                duplicate_targets.add(value.manifest_id)
            targets[value.manifest_id] = value

    def insert(records: dict[str, dict[str, Any]], record: dict[str, Any]) -> None:
        identifier = record["id"]
        if identifier in records and records[identifier] != record:
            msg = f"Conflicting collection declarations for ID {identifier!r}"
            raise ValueError(msg)
        records[identifier] = record

    def discover(obj: object) -> None:
        if isinstance(obj, _Declaration):
            obj = type(obj).model_validate(obj)
        if isinstance(obj, _Card):
            if obj.target.manifest_id in duplicate_targets:
                msg = f"Duplicate manifest_id {obj.target.manifest_id!r} in collection targets"
                raise ValueError(msg)
            if obj.target.manifest_id not in targets:
                msg = f"Card {obj.id!r} target must be a public Manifest in the entrypoint"
                raise ValueError(msg)
            insert(cards, obj.to_plan())
            discover(obj.source)
        elif isinstance(obj, Collector):
            insert(collectors, obj.to_plan())
            discover(obj.task)
        elif isinstance(obj, Task):
            insert(tasks, obj.to_plan())

    for name, obj in vars(module).items():
        if not name.startswith("_"):
            discover(obj)
    if not tasks and not collectors and not cards:
        return None
    return {
        "version": 1,
        "tasks": [tasks[key] for key in sorted(tasks)],
        "collectors": [collectors[key] for key in sorted(collectors)],
        "cards": [cards[key] for key in sorted(cards)],
    }
