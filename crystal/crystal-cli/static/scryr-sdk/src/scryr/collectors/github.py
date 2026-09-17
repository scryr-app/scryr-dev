"""GitHub CLI integrations for repository pull requests and remote workflows."""

from typing import Annotated, Literal

from pydantic import Field

from .common import Identifier, NonEmpty, Schedule, _CollectorConfig, polling

type Repository = Annotated[
    str, Field(pattern=r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$", max_length=256)
]


class GitHubPullRequestCollector(_CollectorConfig):
    """Read bounded pull-request context using the existing gh login."""

    kind: Literal["github_pull_requests"] = "github_pull_requests"
    id: Identifier = "github_pull_requests"
    repository: Repository
    limit: int = Field(default=20, ge=1, le=100)
    schedule: Schedule = Field(default_factory=lambda: polling(180))


class GitHubActionsCollector(_CollectorConfig):
    """Read remote workflow runs; these are repository evidence, not local checks."""

    kind: Literal["github_actions"] = "github_actions"
    id: Identifier = "github_actions"
    repository: Repository
    branch: NonEmpty | None = None
    workflow: NonEmpty | None = None
    limit: int = Field(default=20, ge=1, le=100)
    schedule: Schedule = Field(default_factory=lambda: polling(180))
