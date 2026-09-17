"""Optional resource metrics from explicitly selected Docker targets."""

from typing import Literal

from pydantic import Field

from .common import Identifier, NonEmpty, Schedule, _CollectorConfig, polling


class DockerStatsCollector(_CollectorConfig):
    """Read bounded resource samples from containers in an explicit context."""

    kind: Literal["docker_stats"] = "docker_stats"
    id: Identifier = "docker_stats"
    containers: list[NonEmpty] = Field(min_length=1, max_length=100)
    context: NonEmpty = "default"
    schedule: Schedule = Field(default_factory=lambda: polling(30))
