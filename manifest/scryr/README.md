# Scryr

Scryr is a Python package for defining architecture manifests as typed Pydantic
models. Applications can use it as a library.
## Local collectors and cards

Declare commands, schedules, output contracts, and cards in `index.scry`:

```python
from datetime import timedelta
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field
from scryr import Collector, Command, Every, HistoryCard, JsonOutput, Manifest, StatusCard, Task


class DeploymentSnapshot(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)

    version: str
    state: Literal["healthy", "degraded"]
    ready_instances: int = Field(ge=0)


api = Manifest(name="API", manifest_id="api")
deployment = Task[DeploymentSnapshot](
    id="deployment-status",
    command=Command(argv=["./ops/deployment-status", "--json"]),
    output=JsonOutput(model=DeploymentSnapshot),
    timeout=timedelta(seconds=20),
)
production = Collector[DeploymentSnapshot](
    id="production-deployment",
    task=deployment,
    environment="production",
    schedule=Every(timedelta(seconds=30)),
    stale_after=timedelta(minutes=2),
    retain_for=timedelta(days=30),
)
status = StatusCard[DeploymentSnapshot](
    id="deployment", title="Deployment", source=production, target=api
)
history = HistoryCard[DeploymentSnapshot](
    id="deployment-history", title="Deployment history", source=production, target=api
)
```

`Task[T]` declares an operation and its output type. `Collector[T]` schedules that
operation; `environment` is a descriptive label, not a remote execution target.
`StatusCard[T]`, `TableCard[T]`, and `HistoryCard[T]` display the same persisted
observations without running commands. Card targets require an explicit
`manifest_id` and must be public Manifest objects in the entrypoint. Target IDs
must be unique. All declaration IDs must be stable; conflicting definitions with
one ID fail compilation. Public declarations and their referenced dependencies
are discovered automatically, including tasks referenced only through a card.

The Python SDK exports a versioned collection plan. Rust executes the argument
array directly, validates stdout as JSON, and stores observations. `cwd` defaults
to the manifest directory. Commands must finish without interactive input.
`accepted_exit_codes` defaults to `[0]`; a test reporter can use `[0, 1]` when
both codes produce a valid report. A valid report saying tests failed is distinct
from failure to collect a report. `timeout` defaults to 30 seconds and
`max_output_bytes` to 1 MiB (maximum 16 MiB). Durations must be positive whole
seconds. Timeouts cannot exceed one day.

Specify `ConfigDict(extra="forbid", strict=True)` on output models for an explicit
contract. Rust checks JSON types without Python coercion and never inserts model
defaults, executes validators, or constructs Python output instances. A schema
`default` is an annotation; an omitted optional field stays omitted. Python
validators, serializers, computed fields, recursive schemas, and unsupported
schema keywords are rejected rather than silently ignored. This includes custom
initialization hooks, schema overrides, string transformations, and model-level
string constraints; use `Field(min_length=..., max_length=...)` instead. Simple
string aliases define the JSON field name; alias paths and choices are unsupported.
Declarations are revalidated during export, including mutations after construction.

The portable subset supports primitive JSON types, objects, arrays, enums,
constants, `anyOf`, nonrecursive local `$defs` references, required fields,
additional properties, numeric bounds, string lengths, and array lengths.
Patterns, formats, `multipleOf`, `allOf`, and `oneOf` are unsupported. Integer
values and schema bounds must fit signed or unsigned 64-bit integers; floating
point numbers must be finite. Keep custom
processing inside the declared command and return a portable report. Each card
currently shows a complete report; field projections and custom visualization
plugins are outside this first milestone.

See [`tests/samples/collectors/index.scry`](../tests/samples/collectors/index.scry)
for a runnable, read-only example using `printf`, and the
[CLI documentation](../../README.md) for manual collection and opt-in scheduling.
