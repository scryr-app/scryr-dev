"""Declarative GitHub delivery sources; constructing these models performs no I/O."""

from enum import StrEnum
from typing import Annotated, Literal

from pydantic import AwareDatetime, BaseModel, ConfigDict, Field, model_validator
from pydantic.alias_generators import to_camel

type GitHubName = Annotated[
    str, Field(min_length=1, max_length=100, pattern=r"^[A-Za-z0-9][A-Za-z0-9_.-]*$")
]
type WorkflowPath = Annotated[
    str, Field(pattern=r"^\.github/workflows/[A-Za-z0-9_.-]+\.ya?ml$", max_length=256)
]
type EnvironmentName = Annotated[str, Field(min_length=1, max_length=100)]


class _DeliveryModel(BaseModel):
    model_config = ConfigDict(
        extra="forbid",
        populate_by_name=True,
        serialize_by_alias=True,
        validate_default=True,
        alias_generator=to_camel,
    )


class GitHubRepository(_DeliveryModel):
    """GitHub repository identity; authentication belongs to the collector's gh session."""

    host: str = Field(default="github.com", pattern=r"^[A-Za-z0-9][A-Za-z0-9.-]*$", max_length=253)
    owner: GitHubName
    name: GitHubName


class ActionsBuildSource(_DeliveryModel):
    """Select build runs by exact workflow path and their named artifacts."""

    workflow: WorkflowPath
    artifact_names: list[Annotated[str, Field(min_length=1, max_length=256)]] = Field(
        default_factory=list, alias="artifactNames", max_length=100
    )


class GitHubReleaseSource(_DeliveryModel):
    """Select published releases independently of environment deployments."""

    include_drafts: bool = Field(default=False, alias="includeDrafts")
    include_prereleases: bool = Field(default=False, alias="includePrereleases")


class GitHubDeploymentSource(_DeliveryModel):
    """Select GitHub deployment records; success is a reported deployment outcome."""

    environment: EnvironmentName
    task: str = Field(default="deploy", min_length=1, max_length=100)


class DeliveryReportSchema(StrEnum):
    """Supported, versioned workflow evidence contracts."""

    delivery = "scryr.delivery.v1"
    runtime = "scryr.runtime.v1"
    compatibility = "scryr.compatibility.v1"


class ActionsReportSource(_DeliveryModel):
    """Versioned evidence uploaded by an explicitly selected Actions workflow."""

    workflow: WorkflowPath
    artifact_name: str = Field(alias="artifactName", min_length=1, max_length=256)
    report_schema: DeliveryReportSchema = Field(alias="schema")
    max_age_seconds: int = Field(default=900, alias="maxAgeSeconds", ge=60, le=604800)


class RollbackPolicy(_DeliveryModel):
    """Evidence required to consider a rollback candidate eligible, never a safety guarantee."""

    require_artifact_available: bool = Field(default=True, alias="requireArtifactAvailable")
    require_verified_provenance: bool = Field(default=True, alias="requireVerifiedProvenance")
    require_compatibility_evidence: bool = Field(default=True, alias="requireCompatibilityEvidence")


class DeliveryEnvironment(_DeliveryModel):
    """Environment-specific deployment, runtime and rollback evidence bindings."""

    name: EnvironmentName
    deployments: GitHubDeploymentSource
    runtime: ActionsReportSource | None = None
    compatibility: ActionsReportSource | None = None
    rollback: RollbackPolicy = Field(default_factory=RollbackPolicy)

    @model_validator(mode="after")
    def validate_sources(self) -> DeliveryEnvironment:
        """Prevent accidentally treating another report kind or environment as evidence."""
        if self.name != self.deployments.environment:
            msg = "Deployment environment must match the delivery environment name"
            raise ValueError(msg)
        if self.runtime and self.runtime.report_schema != DeliveryReportSchema.runtime:
            msg = "Runtime sources require scryr.runtime.v1"
            raise ValueError(msg)
        if (
            self.compatibility
            and self.compatibility.report_schema != DeliveryReportSchema.compatibility
        ):
            msg = "Compatibility sources require scryr.compatibility.v1"
            raise ValueError(msg)
        return self


class DeliveryTask(_DeliveryModel):
    """Typed collection settings, executed by Scryr without shell command strings."""

    limit: int = Field(default=20, ge=1, le=100)
    verify_attestations: bool = False


class DeliveryVerificationTask(DeliveryTask):
    """Collection with release asset attestation verification enabled by default."""

    verify_attestations: bool = True


class DeliveryTasks(_DeliveryModel):
    """Named delivery operations owned by the delivery definition."""

    collect: DeliveryTask = Field(default_factory=DeliveryTask)
    verify: DeliveryVerificationTask = Field(default_factory=DeliveryVerificationTask)


class GitHubDelivery(_DeliveryModel):
    """GitHub-backed delivery configuration for a stable Manifest identity."""

    kind: Literal["github"] = "github"
    repository: GitHubRepository
    builds: ActionsBuildSource
    tasks: DeliveryTasks = Field(default_factory=DeliveryTasks)
    releases: GitHubReleaseSource = Field(default_factory=GitHubReleaseSource)
    environments: list[DeliveryEnvironment] = Field(default_factory=list, max_length=50)

    @model_validator(mode="after")
    def validate_environments(self) -> GitHubDelivery:
        """Environment names are unique keys for operational evidence."""
        names = [environment.name for environment in self.environments]
        if len(names) != len(set(names)):
            msg = "Delivery environment names must be unique"
            raise ValueError(msg)
        return self


type ArtifactDigest = Annotated[str, Field(pattern=r"^sha256:[a-f0-9]{64}$")]
type EvidenceIdentity = Annotated[str, Field(min_length=1, max_length=256)]
type CommitRevision = Annotated[str, Field(pattern=r"^(?:[a-fA-F0-9]{40}|[a-fA-F0-9]{64})$")]


class DeliveryLink(_DeliveryModel):
    """Explicit build-to-deployment link published as deployment payload.scryr."""

    report_schema: Literal["scryr.delivery.v1"] = Field(default="scryr.delivery.v1", alias="schema")
    manifest_id: EvidenceIdentity
    environment: EnvironmentName
    deployment_id: EvidenceIdentity | None = None
    build_run_id: EvidenceIdentity
    build_attempt: int = Field(ge=1)
    source_revision: CommitRevision
    artifact_digest: ArtifactDigest
    release_id: EvidenceIdentity | None = None
    configuration_revision: EvidenceIdentity


class DeliveryRuntimeWorkload(_DeliveryModel):
    """An actual probe's observed replica distribution for one workload and digest."""

    name: EvidenceIdentity
    artifact_digest: ArtifactDigest
    replicas: int = Field(ge=1)
    ready_replicas: int = Field(ge=0)

    @model_validator(mode="after")
    def validate_replicas(self) -> DeliveryRuntimeWorkload:
        """Ready replicas cannot exceed the observed replica count."""
        if self.ready_replicas > self.replicas:
            msg = "Ready replicas cannot exceed replicas"
            raise ValueError(msg)
        return self


class DeliveryRuntimeEvidence(_DeliveryModel):
    """Versioned runtime report for upload to a configured Actions artifact."""

    report_schema: Literal["scryr.runtime.v1"] = Field(default="scryr.runtime.v1", alias="schema")
    manifest_id: EvidenceIdentity
    environment: EnvironmentName
    deployment_id: EvidenceIdentity
    configuration_revision: EvidenceIdentity
    schema_revision: EvidenceIdentity
    observed_at: AwareDatetime
    workloads: list[DeliveryRuntimeWorkload] = Field(min_length=1, max_length=1000)


class DeliveryCompatibilityCheck(_DeliveryModel):
    """Result of a named check, never a boolean assertion that rollback is safe."""

    name: EvidenceIdentity
    outcome: Literal["passed", "failed", "unknown"]
    message: str | None = Field(default=None, max_length=4096)


class DeliveryCompatibilityEvidence(_DeliveryModel):
    """Candidate tests bound to the observed environment schema and configuration."""

    report_schema: Literal["scryr.compatibility.v1"] = Field(
        default="scryr.compatibility.v1", alias="schema"
    )
    manifest_id: EvidenceIdentity
    environment: EnvironmentName
    candidate_digest: ArtifactDigest
    current_schema_revision: EvidenceIdentity
    current_configuration_revision: EvidenceIdentity
    candidate_configuration_revision: EvidenceIdentity | None = None
    suite_revision: EvidenceIdentity
    observed_at: AwareDatetime
    checks: list[DeliveryCompatibilityCheck] = Field(min_length=1, max_length=1000)
