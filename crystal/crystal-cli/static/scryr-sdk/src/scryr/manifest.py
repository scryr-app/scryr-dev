"""Manifest data models."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from enum import Enum
from typing import TYPE_CHECKING, Any, ClassVar, Literal, override

from pydantic import (
    AliasChoices,
    BaseModel,
    ConfigDict,
    Field,
    field_serializer,
    model_validator,
)

from .types import (
    CalendarVersion,
    Classification,
    IncrementalVersion,
    Label,
    Markdown,
    SemVer,
    Url,
)

if TYPE_CHECKING:
    import builtins
    from collections.abc import Callable

    from pydantic.main import IncEx

type ForgePrimitive = str | int | float | bool
type ForgeEnvValue = ForgePrimitive | ForgeEnvDirective
type ForgeToolSpec = str | Sequence[str] | ForgeTool
type ForgeTaskRunStep = str | ForgeTaskCommand | ForgeTaskBatch
type ForgeTaskRun = str | Sequence[ForgeTaskRunStep]
type ForgeTaskDependency = str | ForgeTaskCommand
type ForgeTaskDependencyList = ForgeTaskDependency | Sequence[ForgeTaskDependency]
type ForgeTaskSpec = str | Sequence[ForgeTaskRunStep] | ForgeTask

_QUERY_MISSING = object()


class ForgeEnvDirective(BaseModel):
    """A typed mise `[env]` or `[vars]` directive value."""

    model_config = ConfigDict(extra="allow", validate_default=True)

    value: ForgePrimitive | list[str] | None = Field(
        default=None,
        description="Explicit value for the variable or var directive",
    )
    default: str | int | bool | None = Field(
        default=None,
        description="Fallback value used only when no earlier value is set",
    )
    required: str | bool | None = Field(
        default=None,
        description="Required-variable marker or error message",
    )
    path: str | list[str] | None = Field(
        default=None,
        description="Path value or path list for mise path-like directives",
    )
    tools: bool = Field(
        default=False,
        description="Resolve this directive after tool-provided environment values are available",
    )
    redact: bool = Field(default=False, description="Redact this value from mise task output")


class ForgeTool(BaseModel):
    """Detailed mise `[tools]` entry."""

    model_config = ConfigDict(extra="allow", validate_default=True)

    version: str | list[str] = Field(description="Tool version or versions")
    os: str | list[str] | None = Field(default=None, description="Operating systems for this tool")
    depends: list[str] = Field(
        default_factory=list,
        description="Tools that should be installed before this tool",
    )
    install_env: dict[str, ForgePrimitive] = Field(
        default_factory=dict,
        description="Environment variables used while installing this tool",
    )
    postinstall: str | list[str] | None = Field(
        default=None,
        description="Command or commands to run after installing this tool",
    )


class ForgeTaskCommand(BaseModel):
    """Structured mise task reference used in `run`, `depends`, and related fields."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    task: str = Field(description="Task name to run")
    args: list[str] = Field(default_factory=list, description="Arguments passed to the task")
    env: dict[str, ForgePrimitive] = Field(
        default_factory=dict,
        description="Environment variables passed to the task reference",
    )


class ForgeTaskBatch(BaseModel):
    """Parallel task batch used inside a mise task `run` list."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    tasks: list[str] = Field(default_factory=list, description="Task names to run in parallel")


class ForgeTaskConfirm(BaseModel):
    """Detailed mise task confirmation prompt."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    message: str = Field(description="Prompt shown before running the task")
    default: str | bool | None = Field(default=None, description="Default confirmation response")


class ForgeTask(BaseModel):
    """Detailed mise `[tasks.<name>]` entry."""

    model_config = ConfigDict(extra="allow", validate_default=True)

    run: ForgeTaskRun | None = Field(default=None, description="Command or commands to run")
    run_windows: ForgeTaskRun | None = Field(
        default=None,
        description="Windows-specific command or commands to run",
    )
    description: Markdown | str = Field(default=Markdown(""), description="Task description")
    alias: str | list[str] | None = Field(default=None, description="Task alias or aliases")
    depends: ForgeTaskDependencyList | None = Field(
        default=None,
        description="Tasks that must run before this task",
    )
    depends_post: ForgeTaskDependencyList | None = Field(
        default=None,
        description="Tasks that run after this task completes",
    )
    wait_for: ForgeTaskDependencyList | None = Field(
        default=None,
        description="Optional tasks to wait for if already running",
    )
    env: dict[str, ForgeEnvValue] = Field(
        default_factory=dict,
        description="Task-local environment variables",
    )
    tools: dict[str, ForgeToolSpec] = Field(
        default_factory=dict,
        description="Task-local tool versions",
    )
    dir: str | None = Field(default=None, description="Task working directory")
    hide: bool = Field(default=False, description="Hide this task from task listings")
    confirm: str | ForgeTaskConfirm | None = Field(
        default=None,
        description="Confirmation prompt shown before running the task",
    )
    file: str | None = Field(default=None, description="External script file to execute")
    raw: bool = Field(default=False, description="Connect task directly to stdio")
    raw_args: bool = Field(default=False, description="Pass task arguments through verbatim")
    interactive: bool = Field(default=False, description="Run with exclusive interactive stdio")
    sources: str | list[str] | None = Field(default=None, description="Task input files or globs")
    outputs: str | list[str] | None = Field(default=None, description="Task output files or globs")
    shell: str | None = Field(default=None, description="Shell used to run inline task commands")
    quiet: bool = Field(default=False, description="Suppress mise task runner output")
    silent: bool | Literal["stdout", "stderr"] = Field(
        default=False,
        description="Suppress task output",
    )
    output: str | None = Field(default=None, description="Task output style")
    usage: str | None = Field(default=None, description="Task argument usage specification")


class Forge(BaseModel):
    """A top-level Scryr construct wrapping a fully typed mise.toml file."""

    model_config = ConfigDict(extra="forbid", validate_default=True)

    name: Label | str = Field(default=Label(""), description="Display name for this forge")
    description: Markdown | str = Field(default=Markdown(""), description="Short description")
    file_name: str = Field(default="mise.toml", description="Target mise config filename")
    tools: dict[str, ForgeToolSpec] = Field(
        default_factory=dict,
        description="mise `[tools]` table",
    )
    env: dict[str, ForgeEnvValue] = Field(
        default_factory=dict,
        description="mise `[env]` table",
    )
    vars: dict[str, ForgeEnvValue] = Field(
        default_factory=dict,
        description="mise `[vars]` table",
    )
    tasks: dict[str, ForgeTaskSpec] = Field(
        default_factory=dict,
        description="mise `[tasks]` table",
    )
    settings: dict[str, object] = Field(
        default_factory=dict,
        description="mise `[settings]` table",
    )
    plugins: dict[str, str] = Field(
        default_factory=dict,
        description="mise `[plugins]` table",
    )
    redactions: list[str] = Field(
        default_factory=list,
        description="Environment variable patterns that should be redacted",
    )
    raw: dict[str, object] = Field(
        default_factory=dict,
        description="Additional mise TOML sections not yet modeled by Scryr",
    )

    def to_mise_dict(self) -> builtins.dict[str, Any]:
        """Return only the mise.toml content represented by this forge."""
        data = self.model_dump(mode="json", exclude={"name", "description", "file_name", "raw"})
        return {**data, **self.raw}

    def to_dict(self) -> builtins.dict[str, Any]:
        """Return a plain dict (JSON-safe) representation of this forge."""
        import json

        return json.loads(self.model_dump_json())


class Manifest(BaseModel):
    """A data model representing a single Scryr block."""

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )

    manifest_id: str | None = Field(
        default=None,
        min_length=1,
        max_length=256,
        pattern=r"^[A-Za-z0-9][A-Za-z0-9._:/-]*$",
        serialization_alias="manifestId",
        validation_alias=AliasChoices("manifest_id", "manifestId"),
        description="Stable, organization-scoped component identity",
    )

    # Core identity and graph topology.
    name: Label | str = Field(default=Label(""), description="Display name for the component")
    icon: Label | str = Field(default=Label(""), description="Emoji or small icon string")
    classification: Classification = Field(
        default=Classification.public_api,
        description="Block classification for architecture visualization",
    )
    tags: list[Label | str] = Field(
        default_factory=list, description="Arbitrary tags for categorization and filtering"
    )
    connections: list[Manifest] = Field(
        default_factory=list,
        description="Named connections to other components or services",
    )
    forges: list[Label | str] = Field(
        default_factory=list,
        description="Named forge configurations this component uses",
    )

    @field_serializer("classification")
    def _serialize_classification(self, value: Classification) -> str:
        return value.value

    @override
    def dict(
        self,
        *,
        mode: str = "json",
        include: IncEx | None = None,
        exclude: IncEx | None = None,
        context: Any | None = None,
        by_alias: bool | None = None,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        exclude_computed_fields: bool = False,
        round_trip: bool = False,
        warnings: bool | Literal["none", "warn", "error"] = True,
        fallback: Callable[[Any], Any] | None = None,
        serialize_as_any: bool = False,
        polymorphic_serialization: bool | None = None,
    ) -> builtins.dict[str, Any]:
        """Return a JSON-friendly model dump."""
        return super().model_dump(
            mode=mode,
            include=include,
            exclude=exclude,
            context=context,
            by_alias=True if by_alias is None else by_alias,
            exclude_unset=exclude_unset,
            exclude_defaults=exclude_defaults,
            exclude_none=exclude_none,
            exclude_computed_fields=exclude_computed_fields,
            round_trip=round_trip,
            warnings=warnings,
            fallback=fallback,
            serialize_as_any=serialize_as_any,
            polymorphic_serialization=polymorphic_serialization,
        )

    def to_dict(self) -> builtins.dict[str, Any]:
        """Return a plain dict (JSON-safe) representation of this manifest."""
        import json

        return json.loads(self.model_dump_json(by_alias=True))

    def __str__(self) -> str:
        """Return a string representation of the manifest."""
        return f"<{self.name}>"


def _model_field_name(model: type[BaseModel], field_path_part: str) -> str | None:
    """Return the Python field name for a field name or serialization alias."""
    if field_path_part in model.model_fields:
        return field_path_part

    for field_name, field_info in model.model_fields.items():
        if field_info.serialization_alias == field_path_part:
            return field_name

    return None


def _normalize_manifest_query_path(path: str) -> str:
    """Normalize a query field path to Manifest model field names."""
    cleaned_path = path.replace("__", ".").strip(".")
    if not cleaned_path:
        msg = "Manifest query field paths cannot be empty"
        raise ValueError(msg)

    field_name = _model_field_name(Manifest, cleaned_path)
    if field_name is None:
        msg = f"Unknown Manifest query field: {path!r}"
        raise ValueError(msg)
    return field_name


def _flatten_manifest_query_where(
    where: Mapping[str, object],
    prefix: str = "",
) -> dict[str, object]:
    """Flatten nested query dictionaries into dot-separated field paths."""
    flattened: dict[str, object] = {}
    for raw_key, value in where.items():
        key = raw_key.replace("__", ".")
        path = f"{prefix}.{key}" if prefix else key
        if isinstance(value, Mapping):
            nested_value = {
                str(nested_key): nested_item for nested_key, nested_item in value.items()
            }
            flattened.update(_flatten_manifest_query_where(nested_value, path))
        else:
            flattened[path] = value

    return flattened


def _manifest_query_value(value: object) -> object:
    """Normalize values for stable query comparison."""
    if isinstance(value, Enum):
        return str(value)
    if isinstance(value, BaseModel):
        return value.model_dump(mode="json", by_alias=True)
    if isinstance(value, Mapping):
        return {str(key): _manifest_query_value(item) for key, item in value.items()}
    if isinstance(value, Sequence) and not isinstance(value, str):
        return [_manifest_query_value(item) for item in value]
    return (
        str(value)
        if isinstance(value, (Label, Markdown, Url, SemVer, CalendarVersion, IncrementalVersion))
        else value
    )


def _manifest_query_matches(actual: object, expected: object) -> bool:
    """Return whether a Manifest field value satisfies a query condition."""
    normalized_actual = _manifest_query_value(actual)
    normalized_expected = _manifest_query_value(expected)

    if isinstance(normalized_actual, list):
        if isinstance(normalized_expected, list):
            return all(expected_item in normalized_actual for expected_item in normalized_expected)
        return normalized_expected in normalized_actual

    return normalized_actual == normalized_expected


def _manifest_query_path_value(manifest: Manifest, path: str) -> object:
    """Read a normalized dot-path from a Manifest instance."""
    current: object = manifest
    for part in path.split("."):
        if current is None:
            return _QUERY_MISSING
        if isinstance(current, BaseModel):
            current = getattr(current, part, _QUERY_MISSING)
            continue
        if isinstance(current, Mapping):
            current_mapping = {str(key): value for key, value in current.items()}
            current = current_mapping.get(part, _QUERY_MISSING)
            continue
        current = getattr(current, part, _QUERY_MISSING)

    return current


class ManifestQuery(BaseModel):
    """Filter Manifests by a subset of their Pydantic field values."""

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )

    where: dict[str, object] = Field(
        default_factory=dict,
        description=("Manifest field filters such as name, classification, and tags."),
    )

    def __init__(
        self,
        where: Mapping[str, object] | None = None,
        **fields: object,
    ) -> None:
        """Create a query from a where mapping and/or keyword field shorthands."""
        flattened = _flatten_manifest_query_where(where or {})
        flattened.update(_flatten_manifest_query_where(fields))
        normalized = {
            _normalize_manifest_query_path(path): _manifest_query_value(value)
            for path, value in flattened.items()
        }
        super().__init__(where=normalized)

    def matches(self, manifest: Manifest) -> bool:
        """Return whether *manifest* satisfies every query field condition."""
        for path, expected in self.where.items():
            actual = _manifest_query_path_value(manifest, path)
            if actual is _QUERY_MISSING or not _manifest_query_matches(actual, expected):
                return False

        return True

    def filter(self, manifests: Sequence[Manifest]) -> list[Manifest]:
        """Return the manifests that match this query, preserving input order."""
        return [manifest for manifest in manifests if self.matches(manifest)]


class Diagram(BaseModel):
    """A high-level Scryr diagram composed from selected manifests."""

    Query: ClassVar[type[ManifestQuery]] = ManifestQuery

    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
    )

    name: Label | str = Field(default=Label(""), description="Display name for the diagram")
    description: Markdown | str = Field(
        default=Markdown(""),
        description="Short description of the diagram scope",
    )
    manifests: list[Manifest] = Field(
        default_factory=list,
        description="Manifest blocks included in this diagram",
    )
    query: ManifestQuery | None = Field(
        default=None,
        description=(
            "Optional Manifest query used as an alternative to explicitly listing manifests"
        ),
    )

    @model_validator(mode="after")
    def _validate_manifest_selection(self) -> Diagram:
        """Ensure diagrams use either explicit manifests or a query, not both."""
        if self.query is not None and self.manifests:
            msg = "Diagram accepts either manifests or query, not both"
            raise ValueError(msg)
        return self

    def resolved_manifests(self, manifest_pool: Sequence[Manifest]) -> list[Manifest]:
        """Return explicit manifests or query-selected manifests from *manifest_pool*."""
        if self.query is None:
            return self.manifests
        return self.query.filter(manifest_pool)

    @override
    def dict(
        self,
        *,
        mode: str = "json",
        include: IncEx | None = None,
        exclude: IncEx | None = None,
        context: Any | None = None,
        by_alias: bool | None = None,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        exclude_computed_fields: bool = False,
        round_trip: bool = False,
        warnings: bool | Literal["none", "warn", "error"] = True,
        fallback: Callable[[Any], Any] | None = None,
        serialize_as_any: bool = False,
        polymorphic_serialization: bool | None = None,
    ) -> builtins.dict[str, Any]:
        """Return a JSON-friendly model dump."""
        return super().model_dump(
            mode=mode,
            include=include,
            exclude=exclude,
            context=context,
            by_alias=True if by_alias is None else by_alias,
            exclude_unset=exclude_unset,
            exclude_defaults=exclude_defaults,
            exclude_none=exclude_none,
            exclude_computed_fields=exclude_computed_fields,
            round_trip=round_trip,
            warnings=warnings,
            fallback=fallback,
            serialize_as_any=serialize_as_any,
            polymorphic_serialization=polymorphic_serialization,
        )

    def to_dict(
        self,
        manifest_pool: Sequence[Manifest] | None = None,
    ) -> builtins.dict[str, Any]:
        """Return a plain dict (JSON-safe) representation of this diagram."""
        import json

        data = json.loads(self.model_dump_json(by_alias=True))
        if manifest_pool is not None:
            data["manifests"] = [
                manifest.to_dict() for manifest in self.resolved_manifests(manifest_pool)
            ]
        return data
