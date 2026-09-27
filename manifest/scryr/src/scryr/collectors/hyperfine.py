"""Deliberate bounded benchmarks, separate from live service metrics."""

from typing import Literal

from pydantic import Field

from .common import Command, Identifier, NonEmpty, _CollectorConfig


class HyperfineBenchmarkCollector(_CollectorConfig):
    """Measure repeated executions with Hyperfine and an optional named baseline."""

    kind: Literal["hyperfine"] = "hyperfine"
    id: Identifier = "hyperfine"
    command: Command
    warmup: int = Field(default=2, ge=0, le=100)
    runs: int = Field(default=10, ge=2, le=1000)
    baseline: NonEmpty | None = None
