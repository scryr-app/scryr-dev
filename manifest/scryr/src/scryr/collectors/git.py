"""Local Git repository observations without implicit fetches."""

from typing import Literal

from pydantic import Field

from .common import Identifier, ProjectPath, Schedule, _CollectorConfig, polling


class GitStatusCollector(_CollectorConfig):
    """Read branch, commit, dirty files and locally known upstream divergence."""

    kind: Literal["git_status"] = "git_status"
    id: Identifier = "git_status"
    directory: ProjectPath = "."
    schedule: Schedule = Field(default_factory=lambda: polling(15))
