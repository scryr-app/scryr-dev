"""Inert, typed value declarations shared by concrete laptop integrations."""

from __future__ import annotations

from datetime import timedelta
from pathlib import PurePosixPath, PureWindowsPath
from typing import Annotated, Literal

from pydantic import (
    AfterValidator,
    BaseModel,
    ConfigDict,
    Field,
    PlainSerializer,
    field_validator,
    model_validator,
)


def _relative_path(value: str) -> str:
    """Validate a lexical project path without touching the filesystem."""
    if (
        not value.strip()
        or "\x00" in value
        or "\\" in value
        or PurePosixPath(value).is_absolute()
        or PureWindowsPath(value).drive
        or ".." in PurePosixPath(value).parts
    ):
        msg = "Paths must stay within the project and use relative forward-slash paths"
        raise ValueError(msg)
    return value


def _seconds(value: timedelta) -> float:
    """Serialize every duration as seconds on the collector wire."""
    return value.total_seconds()


type ProjectPath = Annotated[str, AfterValidator(_relative_path)]
type NonEmpty = Annotated[str, Field(min_length=1, max_length=4096, pattern=r"^[^\x00]+$")]
type Identifier = Annotated[
    str, Field(min_length=1, max_length=256, pattern=r"^[A-Za-z0-9][A-Za-z0-9._:/-]*$")
]
type Duration = Annotated[
    timedelta, Field(gt=timedelta(0)), PlainSerializer(_seconds, return_type=float)
]
type Debounce = Annotated[
    timedelta, Field(ge=timedelta(0)), PlainSerializer(_seconds, return_type=float)
]


class _Config(BaseModel):
    """Strict declarations; constructing them never performs collection."""

    model_config = ConfigDict(extra="forbid", validate_default=True)


class Schedule(_Config):
    """Choose local triggers; intervals and debounce serialize as seconds."""

    startup: bool = False
    every: Duration | None = None
    watch: list[ProjectPath] = Field(default_factory=list)
    upstream_changed: bool = False
    manual: bool = True
    debounce: Debounce = timedelta(seconds=1)

    @model_validator(mode="after")
    def _require_trigger(self) -> Schedule:
        if not (self.startup or self.every or self.watch or self.upstream_changed or self.manual):
            msg = "Schedule requires at least one trigger"
            raise ValueError(msg)
        return self


class EnvRef(_Config):
    """Reference a machine environment variable without serializing its value."""

    name: str = Field(pattern=r"^[A-Za-z_][A-Za-z0-9_]*$", max_length=128)


class ToolRequirement(_Config):
    """Constrain the integration's fixed executable, without replacing it."""

    version: NonEmpty | None = None


class Command(_Config):
    """An argv-only command used by a benchmark integration."""

    executable: ProjectPath
    args: list[NonEmpty] = Field(default_factory=list)
    cwd: ProjectPath = "."


class SbomRef(_Config):
    """Reference a successful Syft inventory within a manifest or across manifests."""

    collector_id: Identifier
    manifest_id: Identifier | None = None


class LicensePolicy(_Config):
    """License expressions allowed or denied by project policy; unknowns need review."""

    allow: list[NonEmpty] = Field(default_factory=list)
    deny: list[NonEmpty] = Field(default_factory=list)
    unknown: Literal["review", "deny"] = "review"

    @model_validator(mode="after")
    def _no_conflicting_rules(self) -> LicensePolicy:
        if set(self.allow) & set(self.deny):
            msg = "A license expression cannot be both allowed and denied"
            raise ValueError(msg)
        return self


class _CollectorConfig(_Config):
    """Internal common lifecycle fields; use a concrete integration publicly."""

    kind: str
    id: Identifier = "collector"
    schedule: Schedule = Field(default_factory=Schedule)
    timeout: Duration = timedelta(minutes=5)
    freshness: Duration = timedelta(hours=1)
    env: dict[str, EnvRef] = Field(default_factory=dict)
    tool: ToolRequirement | None = None

    @field_validator("env")
    @classmethod
    def _validate_env_names(cls, value: dict[str, EnvRef]) -> dict[str, EnvRef]:
        for name in value:
            EnvRef(name=name)
        return value


def polling(seconds: int) -> Schedule:
    """Return a fresh startup and interval schedule for an integration default."""
    return Schedule(startup=True, every=timedelta(seconds=seconds))
