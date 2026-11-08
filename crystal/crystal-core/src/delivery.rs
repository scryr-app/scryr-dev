//! Typed GitHub delivery configuration and evidence.
//!
//! Deployment success is separate from runtime observations and compatibility.
#![allow(clippy::unused_async_trait_impl)] // async-graphql derives async resolvers for pure values.

use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Values for `DeliveryProvider`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryProvider {
    /// Github.
    #[serde(rename = "github")]
    Github,
}

/// Values for `DeliverySourceCategory`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliverySourceCategory {
    /// Builds.
    #[serde(rename = "builds")]
    Builds,
    /// Artifacts.
    #[serde(rename = "artifacts")]
    Artifacts,
    /// Releases.
    #[serde(rename = "releases")]
    Releases,
    /// Deployments.
    #[serde(rename = "deployments")]
    Deployments,
    /// Runtime.
    #[serde(rename = "runtime")]
    Runtime,
    /// Compatibility.
    #[serde(rename = "compatibility")]
    Compatibility,
    /// Provenance.
    #[serde(rename = "provenance")]
    Provenance,
}

/// Values for `DeliveryEvidenceState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryEvidenceState {
    /// Available.
    #[serde(rename = "available")]
    Available,
    /// Missing.
    #[serde(rename = "missing")]
    Missing,
    /// Stale.
    #[serde(rename = "stale")]
    Stale,
    /// Access denied.
    #[serde(rename = "access_denied")]
    AccessDenied,
    /// Unsupported.
    #[serde(rename = "unsupported")]
    Unsupported,
    /// Error.
    #[serde(rename = "error")]
    Error,
}

/// Values for `DeliveryRunStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryRunStatus {
    /// Queued.
    #[serde(rename = "queued")]
    Queued,
    /// In progress.
    #[serde(rename = "in_progress")]
    InProgress,
    /// Completed.
    #[serde(rename = "completed")]
    Completed,
    /// Waiting.
    #[serde(rename = "waiting")]
    Waiting,
    /// Requested.
    #[serde(rename = "requested")]
    Requested,
    /// Pending.
    #[serde(rename = "pending")]
    Pending,
    /// Unknown.
    #[serde(rename = "unknown")]
    Unknown,
}

/// Values for `DeliveryConclusion`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryConclusion {
    /// Success.
    #[serde(rename = "success")]
    Success,
    /// Failure.
    #[serde(rename = "failure")]
    Failure,
    /// Cancelled.
    #[serde(rename = "cancelled")]
    Cancelled,
    /// Skipped.
    #[serde(rename = "skipped")]
    Skipped,
    /// Timed out.
    #[serde(rename = "timed_out")]
    TimedOut,
    /// Action required.
    #[serde(rename = "action_required")]
    ActionRequired,
    /// Neutral.
    #[serde(rename = "neutral")]
    Neutral,
    /// Stale.
    #[serde(rename = "stale")]
    Stale,
    /// Startup failure.
    #[serde(rename = "startup_failure")]
    StartupFailure,
    /// Unknown.
    #[serde(rename = "unknown")]
    Unknown,
}

/// Values for `DeliveryArtifactKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryArtifactKind {
    /// Actions artifact.
    #[serde(rename = "actions_artifact")]
    ActionsArtifact,
    /// Release asset.
    #[serde(rename = "release_asset")]
    ReleaseAsset,
    /// Oci image.
    #[serde(rename = "oci_image")]
    OciImage,
}

/// Values for `DeliveryVerification`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryVerification {
    /// Verified.
    #[serde(rename = "verified")]
    Verified,
    /// Failed.
    #[serde(rename = "failed")]
    Failed,
    /// Unknown.
    #[serde(rename = "unknown")]
    Unknown,
}

/// Values for `DeliveryDeploymentStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryDeploymentStatus {
    /// Pending.
    #[serde(rename = "pending")]
    Pending,
    /// Queued.
    #[serde(rename = "queued")]
    Queued,
    /// In progress.
    #[serde(rename = "in_progress")]
    InProgress,
    /// Success.
    #[serde(rename = "success")]
    Success,
    /// Failure.
    #[serde(rename = "failure")]
    Failure,
    /// Error.
    #[serde(rename = "error")]
    Error,
    /// Inactive.
    #[serde(rename = "inactive")]
    Inactive,
    /// Unknown.
    #[serde(rename = "unknown")]
    Unknown,
}

/// Values for `DeliveryCheckOutcome`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryCheckOutcome {
    /// Passed.
    #[serde(rename = "passed")]
    Passed,
    /// Failed.
    #[serde(rename = "failed")]
    Failed,
    /// Unknown.
    #[serde(rename = "unknown")]
    Unknown,
}

/// Values for `DeliveryEligibility`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Enum)]
pub enum DeliveryEligibility {
    /// Eligible.
    #[serde(rename = "eligible")]
    Eligible,
    /// Blocked.
    #[serde(rename = "blocked")]
    Blocked,
    /// Unknown.
    #[serde(rename = "unknown")]
    Unknown,
}

/// Typed `GitHubRepository` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitHubRepository {
    /// GitHub host, without scheme.
    pub host: String,
    /// Repository owner.
    pub owner: String,
    /// Repository name.
    pub name: String,
}

/// Typed `ActionsBuildSource` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionsBuildSource {
    /// Workflow path or numeric identifier.
    pub workflow: String,
    /// Selected Actions artifact names; empty selects all.
    pub artifact_names: Vec<String>,
}

/// Typed `GitHubReleaseSource` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitHubReleaseSource {
    /// Include unpublished drafts.
    pub include_drafts: bool,
    /// Include prereleases.
    pub include_prereleases: bool,
}

/// Typed `GitHubDeploymentSource` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitHubDeploymentSource {
    /// Exact GitHub environment name.
    pub environment: String,
    /// Deployment task filter.
    pub task: String,
}

/// Typed `ActionsReportSource` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionsReportSource {
    /// Trusted reporting workflow path.
    pub workflow: String,
    /// Artifact containing the report JSON.
    pub artifact_name: String,
    /// Expected versioned report schema.
    pub schema: String,
    /// Maximum evidence age in seconds.
    pub max_age_seconds: u32,
}

/// Typed `DeliveryRollbackPolicy` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryRollbackPolicy {
    /// Require candidate artifact availability.
    pub require_artifact_available: bool,
    /// Require candidate provenance verification.
    pub require_verified_provenance: bool,
    /// Require scoped compatibility evidence.
    pub require_compatibility_evidence: bool,
}

/// Typed `DeliveryEnvironment` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryEnvironment {
    /// Stable environment identity.
    pub name: String,
    /// Native deployment source.
    pub deployments: GitHubDeploymentSource,
    /// Optional running-artifact evidence.
    pub runtime: Option<ActionsReportSource>,
    /// Optional candidate compatibility evidence.
    pub compatibility: Option<ActionsReportSource>,
    /// Rollback evidence requirements.
    pub rollback: DeliveryRollbackPolicy,
}

/// Typed collection options declared inside a delivery definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct DeliveryTask {
    /// Maximum recent records requested from each source, between one and 100.
    pub limit: u32,
    /// Download release assets and verify their build-workflow attestations.
    pub verify_attestations: bool,
}

impl Default for DeliveryTask {
    fn default() -> Self {
        Self {
            limit: 20,
            verify_attestations: false,
        }
    }
}

/// Built-in delivery actions with declarative, typed options rather than shell commands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct DeliveryTasks {
    /// Collect native delivery records and configured workflow reports.
    pub collect: DeliveryTask,
    /// Collect records and verify release-asset attestations by default.
    #[serde(deserialize_with = "deserialize_verify_task")]
    pub verify: DeliveryTask,
}

impl Default for DeliveryTasks {
    fn default() -> Self {
        Self {
            collect: DeliveryTask::default(),
            verify: DeliveryTask {
                verify_attestations: true,
                ..DeliveryTask::default()
            },
        }
    }
}

/// Preserve verify-task defaults when only some of its options are supplied.
fn deserialize_verify_task<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<DeliveryTask, D::Error> {
    let mut value = serde_json::Value::deserialize(deserializer)?;
    let object = value.as_object_mut().ok_or_else(|| {
        <D::Error as serde::de::Error>::custom("delivery verify task must be an object")
    })?;
    object
        .entry("verifyAttestations")
        .or_insert(serde_json::Value::Bool(true));
    serde_json::from_value(value).map_err(<D::Error as serde::de::Error>::custom)
}

/// Typed `GitHubDelivery` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitHubDelivery {
    /// Provider discriminator.
    pub kind: DeliveryProvider,
    /// Repository identity.
    pub repository: GitHubRepository,
    /// Build source selection.
    pub builds: ActionsBuildSource,
    /// Release source selection.
    pub releases: GitHubReleaseSource,
    /// Configured deployment environments.
    pub environments: Vec<DeliveryEnvironment>,
    /// Named, typed collection actions; defaults preserve legacy definitions.
    #[serde(default)]
    pub tasks: DeliveryTasks,
}

/// Typed `DeliverySourceState` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliverySourceState {
    /// Evidence category.
    pub category: DeliverySourceCategory,
    /// Environment when the source is scoped.
    pub environment: Option<String>,
    /// Collection result.
    pub state: DeliveryEvidenceState,
    /// Non-sensitive diagnostic.
    pub message: Option<String>,
}

/// Typed `DeliveryJob` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryJob {
    /// GitHub job ID.
    pub id: String,
    /// Job display name.
    pub name: String,
    /// Execution state.
    pub status: DeliveryRunStatus,
    /// Completed outcome.
    pub conclusion: Option<DeliveryConclusion>,
    /// Job log URL.
    pub url: String,
}

/// Typed `DeliveryBuild` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryBuild {
    /// GitHub run ID.
    pub id: String,
    /// Workflow attempt number.
    pub attempt: u32,
    /// Workflow path.
    pub workflow: String,
    /// Run display name.
    pub name: String,
    /// Workflow head commit, not inferred artifact provenance.
    pub commit_sha: String,
    /// Execution state.
    pub status: DeliveryRunStatus,
    /// Completed outcome.
    pub conclusion: Option<DeliveryConclusion>,
    /// Provider creation time.
    pub created_at: DateTime<Utc>,
    /// Provider update time.
    pub updated_at: DateTime<Utc>,
    /// Workflow run URL.
    pub url: String,
    /// Jobs for this run attempt.
    pub jobs: Vec<DeliveryJob>,
}

/// Typed `DeliveryArtifact` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryArtifact {
    /// Provider artifact/asset identity.
    pub id: String,
    /// Artifact storage kind.
    pub kind: DeliveryArtifactKind,
    /// Artifact display name.
    pub name: String,
    /// Digest of this artifact, distinct from contained files.
    pub digest: Option<String>,
    /// Producing workflow run when provided.
    pub run_id: Option<String>,
    /// Owning GitHub release when provided.
    pub release_id: Option<String>,
    /// Provider reports artifact expired.
    pub expired: bool,
    /// Artifact reference URL.
    pub url: Option<String>,
    /// Verified cryptographic provenance, never inferred from build success.
    pub provenance: DeliveryVerification,
}

/// Typed `DeliveryRelease` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryRelease {
    /// GitHub release ID.
    pub id: String,
    /// Release tag.
    pub tag: String,
    /// Display title.
    pub name: Option<String>,
    /// Publication time.
    pub published_at: Option<DateTime<Utc>>,
    /// Whether release is a draft.
    pub draft: bool,
    /// Whether release is a prerelease.
    pub prerelease: bool,
    /// Release URL.
    pub url: String,
    /// Explicit release asset identities.
    pub asset_ids: Vec<String>,
}

/// Typed `DeliveryDeployment` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryDeployment {
    /// Append-only provider status history, newest first.
    #[serde(default)]
    pub statuses: Vec<DeliveryDeploymentEvent>,
    /// GitHub deployment ID.
    pub id: String,
    /// Provider environment name.
    pub environment: String,
    /// Requested deployment revision.
    pub commit_sha: String,
    /// Latest reported status.
    pub status: DeliveryDeploymentStatus,
    /// Deployment creation time.
    pub created_at: DateTime<Utc>,
    /// Latest status time.
    pub updated_at: DateTime<Utc>,
    /// Environment or deployment log URL.
    pub url: Option<String>,
}

/// Typed `DeliveryLink` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryLink {
    /// Must be scryr.delivery.v1.
    pub schema: String,
    /// Stable Scryr manifest identity.
    pub manifest_id: String,
    /// Configured environment name.
    pub environment: String,
    /// Explicit GitHub deployment ID.
    pub deployment_id: String,
    /// Explicit producing build run ID.
    pub build_run_id: String,
    /// Producing run attempt.
    pub build_attempt: u32,
    /// Actual source revision reported by the build.
    pub source_revision: String,
    /// Deployed artifact content digest.
    pub artifact_digest: String,
    /// Explicit GitHub release ID.
    pub release_id: Option<String>,
    /// Deployed configuration identity.
    pub configuration_revision: String,
}

/// Typed `DeliveryRuntimeWorkload` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryRuntimeWorkload {
    /// Workload/container identity.
    pub name: String,
    /// Observed content digest.
    pub artifact_digest: String,
    /// Observed replica count.
    pub replicas: u32,
    /// Ready replicas for this digest.
    pub ready_replicas: u32,
}

/// Typed `DeliveryRuntimeEvidence` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryRuntimeEvidence {
    /// Must be scryr.runtime.v1.
    pub schema: String,
    /// Stable Scryr manifest identity.
    pub manifest_id: String,
    /// Configured environment name.
    pub environment: String,
    /// Deployment the probe identifies.
    pub deployment_id: String,
    /// Observed configuration identity.
    pub configuration_revision: String,
    /// Observed database schema identity.
    pub schema_revision: String,
    /// Time of actual probe.
    pub observed_at: DateTime<Utc>,
    /// Observed workload digest distribution.
    pub workloads: Vec<DeliveryRuntimeWorkload>,
}

/// Typed `DeliveryCompatibilityCheck` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryCompatibilityCheck {
    /// Performed compatibility check.
    pub name: String,
    /// Check result.
    pub outcome: DeliveryCheckOutcome,
    /// Human-readable result detail.
    pub message: Option<String>,
}

/// Typed `DeliveryCompatibilityEvidence` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryCompatibilityEvidence {
    /// Must be scryr.compatibility.v1.
    pub schema: String,
    /// Stable Scryr manifest identity.
    pub manifest_id: String,
    /// Configured environment name.
    pub environment: String,
    /// Exact candidate artifact tested.
    pub candidate_digest: String,
    /// Candidate configuration tested; omission only applies when current and candidate configurations match.
    #[serde(default)]
    pub candidate_configuration_revision: Option<String>,
    /// Schema identity against which the candidate was tested.
    pub current_schema_revision: String,
    /// Configuration identity against which the candidate was tested.
    pub current_configuration_revision: String,
    /// Version of the checks that were executed.
    pub suite_revision: String,
    /// Actual check completion time.
    pub observed_at: DateTime<Utc>,
    /// Individual check outcomes.
    pub checks: Vec<DeliveryCompatibilityCheck>,
}

/// Typed `DeliverySnapshot` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliverySnapshot {
    /// Exact declaration used by the collector; prevents reusing evidence after source changes.
    #[serde(default)]
    pub configuration: Option<GitHubDelivery>,
    /// GitHub host.
    pub host: String,
    /// Repository owner/name.
    pub repository: String,
    /// Collection snapshot time.
    pub observed_at: DateTime<Utc>,
    /// Collection status per source.
    pub sources: Vec<DeliverySourceState>,
    /// Observed build runs.
    pub builds: Vec<DeliveryBuild>,
    /// Observed artifact inventory.
    pub artifacts: Vec<DeliveryArtifact>,
    /// Observed published releases.
    pub releases: Vec<DeliveryRelease>,
    /// Observed deployment history.
    pub deployments: Vec<DeliveryDeployment>,
    /// Explicit source/build/artifact/deployment links.
    pub links: Vec<DeliveryLink>,
    /// Workflow-published runtime observations.
    pub runtime: Vec<DeliveryRuntimeEvidence>,
    /// Workflow-published compatibility observations.
    pub compatibility: Vec<DeliveryCompatibilityEvidence>,
}

/// Typed `DeliveryAssessment` record.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryAssessment {
    /// Configured environment name.
    pub environment: String,
    /// Freshness and availability of runtime evidence.
    pub runtime_state: DeliveryEvidenceState,
    /// Whether every observed workload matches the deployment link.
    pub running_matches_deployment: Option<bool>,
    /// Previous successful deployment candidate.
    pub candidate_deployment_id: Option<String>,
    /// Explicit candidate artifact digest.
    pub candidate_digest: Option<String>,
    /// Whether configured checks establish rollback eligibility.
    pub eligibility: DeliveryEligibility,
    /// Assessment checks with explicit unknown outcomes.
    pub checks: Vec<DeliveryCompatibilityCheck>,
}

/// One GitHub deployment status event.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliveryDeploymentEvent {
    /// Reported deployment state.
    pub status: DeliveryDeploymentStatus,
    /// Time reported by GitHub.
    pub created_at: DateTime<Utc>,
    /// Environment or log URL.
    pub url: Option<String>,
}

impl DeliverySnapshot {
    /// Start an empty snapshot at the current collection time.
    #[must_use]
    pub fn new(host: String, repository: String) -> Self {
        Self {
            configuration: None,
            host,
            repository,
            observed_at: Utc::now(),
            sources: vec![],
            builds: vec![],
            artifacts: vec![],
            releases: vec![],
            deployments: vec![],
            links: vec![],
            runtime: vec![],
            compatibility: vec![],
        }
    }
}

fn identity(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256
}

fn digest(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|c| c.is_ascii_hexdigit()))
}

fn revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|c| c.is_ascii_hexdigit())
}

fn safe_url(value: &str) -> bool {
    value.starts_with("https://") && !value.chars().any(char::is_control)
}

fn workflow(value: &str) -> bool {
    value
        .strip_prefix(".github/workflows/")
        .is_some_and(|file| {
            matches!(
                std::path::Path::new(file)
                    .extension()
                    .and_then(|ext| ext.to_str()),
                Some("yml" | "yaml")
            ) && file
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
        && value.len() <= 256
}

impl GitHubRepository {
    /// Validate repository components before they are used in provider API paths.
    /// # Errors
    /// Rejects invalid host names and repository path components.
    pub fn validate(&self) -> Result<(), String> {
        let segment = |v: &str| {
            identity(v)
                && v.starts_with(|c: char| c.is_ascii_alphanumeric())
                && v.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
                && !matches!(v, "." | "..")
        };
        if !segment(&self.owner)
            || !segment(&self.name)
            || !identity(&self.host)
            || self
                .host
                .split('.')
                .any(|part| !segment(part) || part.contains('_'))
        {
            return Err("invalid GitHub host or owner/repository".into());
        }
        Ok(())
    }
}

impl GitHubDelivery {
    /// Validate a declaration without contacting GitHub or executing project code.
    /// # Errors
    /// Rejects malformed workflow selectors, duplicate environments, or invalid bounds.
    pub fn validate(&self) -> Result<(), String> {
        self.repository.validate()?;
        if [&self.tasks.collect, &self.tasks.verify]
            .iter()
            .any(|task| !(1..=100).contains(&task.limit))
        {
            return Err("delivery task limits must be between 1 and 100".into());
        }
        if !workflow(&self.builds.workflow)
            || self.environments.len() > 50
            || self
                .builds
                .artifact_names
                .iter()
                .any(|name| !identity(name))
        {
            return Err("delivery requires a workflow path and at most 50 environments".into());
        }
        let mut names = std::collections::HashSet::new();
        for environment in &self.environments {
            if !identity(&environment.name)
                || !names.insert(&environment.name)
                || environment.name != environment.deployments.environment
                || !identity(&environment.deployments.task)
            {
                return Err(
                    "delivery environments must be unique and match deployment selectors".into(),
                );
            }
            for (source, schema) in [
                (&environment.runtime, "scryr.runtime.v1"),
                (&environment.compatibility, "scryr.compatibility.v1"),
            ] {
                if let Some(source) = source
                    && (!workflow(&source.workflow)
                        || !identity(&source.artifact_name)
                        || source.schema != schema
                        || !(60..=604_800).contains(&source.max_age_seconds))
                {
                    return Err("invalid delivery report source or freshness bound".into());
                }
            }
        }
        Ok(())
    }
}

impl DeliverySnapshot {
    /// Validate evidence boundaries and explicit relationships.
    /// # Errors
    /// Rejects malformed identities, digests, timestamps, and contradictory references.
    pub fn validate(&self) -> Result<(), String> {
        let (owner, name) = self
            .repository
            .split_once('/')
            .ok_or("repository must be owner/name")?;
        GitHubRepository {
            host: self.host.clone(),
            owner: owner.into(),
            name: name.into(),
        }
        .validate()?;
        if let Some(configuration) = &self.configuration {
            configuration.validate()?;
            if configuration.repository.host != self.host
                || format!(
                    "{}/{}",
                    configuration.repository.owner, configuration.repository.name
                ) != self.repository
            {
                return Err("snapshot configuration repository does not match evidence".into());
            }
        }
        let now = Utc::now() + chrono::Duration::seconds(60);
        if self.observed_at > now
            || [
                self.builds.len(),
                self.artifacts.len(),
                self.releases.len(),
                self.deployments.len(),
                self.links.len(),
                self.runtime.len(),
                self.compatibility.len(),
            ]
            .iter()
            .any(|n| *n > 1000)
        {
            return Err("delivery snapshot exceeds bounds or is from the future".into());
        }
        self.validate_native(now)?;
        self.validate_workflow_evidence(now)
    }

    fn validate_native(&self, now: DateTime<Utc>) -> Result<(), String> {
        let mut ids = std::collections::HashSet::new();
        for build in &self.builds {
            if !identity(&build.id)
                || build.attempt == 0
                || !revision(&build.commit_sha)
                || !ids.insert((&build.id, build.attempt))
                || !safe_url(&build.url)
                || build.updated_at < build.created_at
                || build.updated_at > now
                || build.jobs.len() > 1000
                || !workflow(&build.workflow)
            {
                return Err("invalid or duplicate delivery build".into());
            }
            for job in &build.jobs {
                if !identity(&job.id) || !identity(&job.name) || !safe_url(&job.url) {
                    return Err("invalid delivery job".into());
                }
            }
        }
        let mut ids = std::collections::HashSet::new();
        for artifact in &self.artifacts {
            if !identity(&artifact.id)
                || !identity(&artifact.name)
                || !ids.insert(format!("{:?}:{}", artifact.kind, artifact.id))
                || artifact.digest.as_deref().is_some_and(|v| !digest(v))
                || artifact.url.as_deref().is_some_and(|v| !safe_url(v))
                || (artifact.provenance == DeliveryVerification::Verified
                    && artifact.digest.is_none())
            {
                return Err("invalid or duplicate delivery artifact".into());
            }
        }
        let mut ids = std::collections::HashSet::new();
        for release in &self.releases {
            if !identity(&release.id)
                || !identity(&release.tag)
                || !ids.insert(&release.id)
                || !safe_url(&release.url)
                || release.published_at.is_some_and(|v| v > now)
            {
                return Err("invalid or duplicate delivery release".into());
            }
        }
        let mut ids = std::collections::HashSet::new();
        for deployment in &self.deployments {
            if !identity(&deployment.id)
                || !identity(&deployment.environment)
                || !revision(&deployment.commit_sha)
                || !ids.insert(&deployment.id)
                || deployment.updated_at < deployment.created_at
                || deployment.updated_at > now
                || deployment.url.as_deref().is_some_and(|v| !safe_url(v))
                || deployment.statuses.iter().any(|event| {
                    event.created_at > now || event.url.as_deref().is_some_and(|v| !safe_url(v))
                })
            {
                return Err("invalid or duplicate delivery deployment".into());
            }
        }
        Ok(())
    }

    fn validate_workflow_evidence(&self, now: DateTime<Utc>) -> Result<(), String> {
        let mut linked = std::collections::HashSet::new();
        for link in &self.links {
            crate::reports::validate_manifest_id(&link.manifest_id)?;
            if link.schema != "scryr.delivery.v1"
                || !digest(&link.artifact_digest)
                || !identity(&link.environment)
                || !identity(&link.configuration_revision)
                || !identity(&link.build_run_id)
                || link.build_attempt == 0
                || !revision(&link.source_revision)
                || !linked.insert(&link.deployment_id)
                || !self
                    .deployments
                    .iter()
                    .filter(|d| d.id == link.deployment_id)
                    .any(|d| d.environment == link.environment)
            {
                return Err("invalid or conflicting delivery link".into());
            }
        }
        for runtime in &self.runtime {
            crate::reports::validate_manifest_id(&runtime.manifest_id)?;
            if runtime.schema != "scryr.runtime.v1"
                || !identity(&runtime.environment)
                || !identity(&runtime.deployment_id)
                || !identity(&runtime.configuration_revision)
                || !identity(&runtime.schema_revision)
                || runtime.observed_at > now
                || runtime.workloads.is_empty()
                || runtime.workloads.len() > 1000
                || runtime.workloads.iter().any(|w| {
                    !identity(&w.name)
                        || !digest(&w.artifact_digest)
                        || w.replicas == 0
                        || w.ready_replicas > w.replicas
                })
            {
                return Err("invalid runtime evidence".into());
            }
        }
        for compatibility in &self.compatibility {
            crate::reports::validate_manifest_id(&compatibility.manifest_id)?;
            if compatibility.schema != "scryr.compatibility.v1"
                || !identity(&compatibility.environment)
                || !digest(&compatibility.candidate_digest)
                || compatibility
                    .candidate_configuration_revision
                    .as_deref()
                    .is_some_and(|v| !identity(v))
                || !identity(&compatibility.current_schema_revision)
                || !identity(&compatibility.current_configuration_revision)
                || !identity(&compatibility.suite_revision)
                || compatibility.observed_at > now
                || compatibility.checks.is_empty()
                || compatibility.checks.len() > 1000
                || compatibility.checks.iter().any(|c| !identity(&c.name))
            {
                return Err("invalid compatibility evidence".into());
            }
        }
        Ok(())
    }

    /// Ensure workflow-published evidence belongs to the receiving manifest.
    /// # Errors
    /// Rejects evidence addressed to another component.
    pub fn validate_for_manifest(&self, manifest_id: &str) -> Result<(), String> {
        self.validate()?;
        if self.links.iter().any(|v| v.manifest_id != manifest_id)
            || self.runtime.iter().any(|v| v.manifest_id != manifest_id)
            || self
                .compatibility
                .iter()
                .any(|v| v.manifest_id != manifest_id)
        {
            return Err("delivery evidence manifestId does not match receiving manifest".into());
        }
        Ok(())
    }
}

fn check(name: &str, outcome: DeliveryCheckOutcome, message: &str) -> DeliveryCompatibilityCheck {
    DeliveryCompatibilityCheck {
        name: name.into(),
        outcome,
        message: Some(message.into()),
    }
}

fn fresh(time: DateTime<Utc>, now: DateTime<Utc>, seconds: u32) -> bool {
    time <= now + chrono::Duration::seconds(60)
        && now.signed_duration_since(time).num_seconds() <= i64::from(seconds)
}

impl GitHubDelivery {
    /// Evaluate evidence at read time so cached observations cannot stay fresh forever.
    #[must_use]
    pub fn assess(
        &self,
        snapshot: Option<&DeliverySnapshot>,
        now: DateTime<Utc>,
    ) -> Vec<DeliveryAssessment> {
        self.environments
            .iter()
            .map(|environment| assess_environment(environment, snapshot, now))
            .collect()
    }
}

fn assess_environment(
    environment: &DeliveryEnvironment,
    snapshot: Option<&DeliverySnapshot>,
    now: DateTime<Utc>,
) -> DeliveryAssessment {
    let mut result = DeliveryAssessment {
        environment: environment.name.clone(),
        runtime_state: DeliveryEvidenceState::Missing,
        running_matches_deployment: None,
        candidate_deployment_id: None,
        candidate_digest: None,
        eligibility: DeliveryEligibility::Unknown,
        checks: vec![],
    };
    let Some(snapshot) = snapshot else {
        result.checks.push(check(
            "Delivery evidence",
            DeliveryCheckOutcome::Unknown,
            "No GitHub snapshot has been collected",
        ));
        return result;
    };
    result
        .checks
        .push(if fresh(snapshot.observed_at, now, 900) {
            check(
                "Snapshot freshness",
                DeliveryCheckOutcome::Passed,
                "GitHub inventory was collected within 15 minutes",
            )
        } else {
            check(
                "Snapshot freshness",
                DeliveryCheckOutcome::Unknown,
                "GitHub inventory is stale; collect again before assessing rollback",
            )
        });
    let runtime = assess_runtime(environment, snapshot, now, &mut result);
    let current = current_deployment(environment, snapshot);
    assess_operations(environment, snapshot, current, runtime, &mut result);
    let candidate = current.and_then(|current| {
        snapshot
            .deployments
            .iter()
            .filter(|d| d.environment == environment.name)
            .filter(|d| {
                d.id != current.id
                    && d.created_at < current.created_at
                    && (d.status == DeliveryDeploymentStatus::Success
                        || (d.status == DeliveryDeploymentStatus::Inactive
                            && d.statuses
                                .iter()
                                .any(|s| s.status == DeliveryDeploymentStatus::Success)))
            })
            .max_by_key(|d| (d.created_at, &d.id))
    });
    let link = candidate.and_then(|candidate| {
        snapshot
            .links
            .iter()
            .find(|l| l.deployment_id == candidate.id)
    });
    result.candidate_deployment_id = candidate.map(|d| d.id.clone());
    result.candidate_digest = link.map(|l| l.artifact_digest.clone());
    result.checks.push(if link.is_some() {
        check(
            "Candidate identity",
            DeliveryCheckOutcome::Passed,
            "Previous successful deployment explicitly identifies the candidate artifact",
        )
    } else {
        check(
            "Candidate identity",
            DeliveryCheckOutcome::Unknown,
            "Previous successful deployment requires an explicit artifact link",
        )
    });
    assess_artifact(environment, snapshot, link, now, &mut result);
    assess_compatibility(environment, snapshot, link, runtime, now, &mut result);
    result.eligibility = if result
        .checks
        .iter()
        .any(|c| c.outcome == DeliveryCheckOutcome::Failed)
    {
        DeliveryEligibility::Blocked
    } else if result
        .checks
        .iter()
        .any(|c| c.outcome == DeliveryCheckOutcome::Unknown)
    {
        DeliveryEligibility::Unknown
    } else {
        DeliveryEligibility::Eligible
    };
    result
}

fn assess_operations(
    environment: &DeliveryEnvironment,
    snapshot: &DeliverySnapshot,
    current: Option<&DeliveryDeployment>,
    runtime: Option<&DeliveryRuntimeEvidence>,
    result: &mut DeliveryAssessment,
) {
    let outcome = operation_check(environment, snapshot, current, runtime);
    if outcome.outcome == DeliveryCheckOutcome::Unknown
        && result.running_matches_deployment == Some(true)
    {
        result.running_matches_deployment = None;
        if let Some(runtime_check) = result
            .checks
            .iter_mut()
            .find(|c| c.name == "Current runtime")
        {
            *runtime_check = check(
                "Current runtime",
                DeliveryCheckOutcome::Unknown,
                "A newer deployment operation prevents establishing the current runtime state",
            );
        }
    }
    result.checks.push(outcome);
}

fn operation_check(
    environment: &DeliveryEnvironment,
    snapshot: &DeliverySnapshot,
    current: Option<&DeliveryDeployment>,
    runtime: Option<&DeliveryRuntimeEvidence>,
) -> DeliveryCompatibilityCheck {
    let Some(current) = current else {
        return check(
            "Environment operation",
            DeliveryCheckOutcome::Unknown,
            "No successful deployment establishes the current environment baseline",
        );
    };
    let newer = snapshot
        .deployments
        .iter()
        .filter(|d| d.environment == environment.name)
        .filter(|d| d.id != current.id)
        .filter(|d| d.created_at >= current.created_at || d.updated_at >= current.updated_at);
    for deployment in newer {
        if matches!(
            deployment.status,
            DeliveryDeploymentStatus::Pending
                | DeliveryDeploymentStatus::Queued
                | DeliveryDeploymentStatus::InProgress
                | DeliveryDeploymentStatus::Unknown
        ) {
            return check(
                "Environment operation",
                DeliveryCheckOutcome::Unknown,
                "A newer deployment is active or has an unknown outcome; wait for it to settle",
            );
        }
        if matches!(
            deployment.status,
            DeliveryDeploymentStatus::Failure | DeliveryDeploymentStatus::Error
        ) && runtime.is_none_or(|probe| {
            probe.observed_at < deployment.updated_at.max(deployment.created_at)
        }) {
            return check(
                "Environment operation",
                DeliveryCheckOutcome::Unknown,
                "A runtime probe after the newer failed deployment is required to establish what remains running",
            );
        }
    }
    check(
        "Environment operation",
        DeliveryCheckOutcome::Passed,
        "No newer active deployment exists and runtime evidence covers newer failed operations",
    )
}

fn current_deployment<'a>(
    environment: &DeliveryEnvironment,
    snapshot: &'a DeliverySnapshot,
) -> Option<&'a DeliveryDeployment> {
    snapshot
        .deployments
        .iter()
        .filter(|d| {
            d.environment == environment.name && d.status == DeliveryDeploymentStatus::Success
        })
        .max_by_key(|d| (d.created_at, &d.id))
}

fn assess_runtime<'a>(
    environment: &DeliveryEnvironment,
    snapshot: &'a DeliverySnapshot,
    now: DateTime<Utc>,
    result: &mut DeliveryAssessment,
) -> Option<&'a DeliveryRuntimeEvidence> {
    let runtime = snapshot
        .runtime
        .iter()
        .filter(|v| v.environment == environment.name)
        .max_by_key(|v| v.observed_at);
    if let Some(source) = snapshot.sources.iter().find(|s| {
        s.category == DeliverySourceCategory::Runtime
            && s.environment.as_ref() == Some(&environment.name)
    }) {
        result.runtime_state = source.state;
    }
    let runtime = match (runtime, &environment.runtime) {
        (Some(runtime), Some(source))
            if fresh(runtime.observed_at, now, source.max_age_seconds) =>
        {
            result.runtime_state = DeliveryEvidenceState::Available;
            Some(runtime)
        }
        (Some(_), Some(_)) => {
            result.runtime_state = DeliveryEvidenceState::Stale;
            None
        }
        _ => {
            if result.runtime_state == DeliveryEvidenceState::Available {
                result.runtime_state = DeliveryEvidenceState::Missing;
            }
            None
        }
    };
    let current = current_deployment(environment, snapshot);
    let current_link =
        current.and_then(|d| snapshot.links.iter().find(|l| l.deployment_id == d.id));
    if let (Some(current), Some(link), Some(runtime)) = (current, current_link, runtime)
        && runtime.deployment_id == current.id
        && runtime.observed_at >= current.updated_at
    {
        let matches = runtime.configuration_revision == link.configuration_revision
            && runtime.workloads.iter().all(|w| {
                w.artifact_digest == link.artifact_digest && w.ready_replicas == w.replicas
            });
        result.running_matches_deployment = Some(matches);
    }
    result.checks.push(match result.running_matches_deployment {
        Some(true) => check(
            "Current runtime",
            DeliveryCheckOutcome::Passed,
            "Fresh probe matches the recorded deployment and all observed replicas are ready",
        ),
        Some(false) => check(
            "Current runtime",
            DeliveryCheckOutcome::Failed,
            "Observed artifacts, configuration, or readiness differ from the deployment",
        ),
        None => check(
            "Current runtime",
            DeliveryCheckOutcome::Unknown,
            "A fresh runtime probe and explicit deployment link are required",
        ),
    });
    runtime
}

fn assess_artifact(
    environment: &DeliveryEnvironment,
    snapshot: &DeliverySnapshot,
    link: Option<&DeliveryLink>,
    now: DateTime<Utc>,
    result: &mut DeliveryAssessment,
) {
    let artifacts: Vec<_> = snapshot
        .artifacts
        .iter()
        .filter(|artifact| {
            link.is_some_and(|link| artifact.digest.as_ref() == Some(&link.artifact_digest))
        })
        .collect();
    if environment.rollback.require_artifact_available {
        let source_available = snapshot.sources.iter().any(|s| {
            s.category == DeliverySourceCategory::Artifacts
                && s.state == DeliveryEvidenceState::Available
        }) && fresh(snapshot.observed_at, now, 900);
        result.checks.push(
            if source_available && artifacts.iter().any(|a| !a.expired) {
                check(
                    "Artifact available",
                    DeliveryCheckOutcome::Passed,
                    "Candidate content is available in at least one fresh artifact inventory entry",
                )
            } else if source_available && !artifacts.is_empty() {
                check(
                    "Artifact available",
                    DeliveryCheckOutcome::Failed,
                    "All observed copies of the candidate artifact have expired",
                )
            } else {
                check(
                    "Artifact available",
                    DeliveryCheckOutcome::Unknown,
                    "Candidate availability has not been established by a fresh inventory",
                )
            },
        );
    }
    if environment.rollback.require_verified_provenance {
        result.checks.push(
            if artifacts
                .iter()
                .any(|a| a.provenance == DeliveryVerification::Verified)
            {
                check(
                    "Provenance",
                    DeliveryCheckOutcome::Passed,
                    "Candidate content has cryptographically verified provenance",
                )
            } else if artifacts
                .iter()
                .any(|a| a.provenance == DeliveryVerification::Failed)
            {
                check(
                    "Provenance",
                    DeliveryCheckOutcome::Failed,
                    "Candidate provenance verification failed",
                )
            } else {
                check(
                    "Provenance",
                    DeliveryCheckOutcome::Unknown,
                    "Candidate provenance has not been verified",
                )
            },
        );
    }
}

fn assess_compatibility(
    environment: &DeliveryEnvironment,
    snapshot: &DeliverySnapshot,
    link: Option<&DeliveryLink>,
    runtime: Option<&DeliveryRuntimeEvidence>,
    now: DateTime<Utc>,
    result: &mut DeliveryAssessment,
) {
    if environment.rollback.require_compatibility_evidence {
        let compatibility = link.zip(runtime).and_then(|(link, runtime)| {
            snapshot
                .compatibility
                .iter()
                .filter(|c| {
                    c.environment == environment.name
                        && c.candidate_digest == link.artifact_digest
                        && c.current_schema_revision == runtime.schema_revision
                        && c.current_configuration_revision == runtime.configuration_revision
                        && c.candidate_configuration_revision
                            .as_deref()
                            .unwrap_or(&runtime.configuration_revision)
                            == link.configuration_revision
                })
                .max_by_key(|c| c.observed_at)
        });
        if let (Some(evidence), Some(source)) = (compatibility, &environment.compatibility)
            && fresh(evidence.observed_at, now, source.max_age_seconds)
        {
            result.checks.extend(evidence.checks.clone());
        } else {
            result.checks.push(check(
                "Migration and configuration compatibility",
                DeliveryCheckOutcome::Unknown,
                "Fresh candidate tests against the observed schema and configuration are required",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn delivery_task_defaults_preserve_legacy_and_partial_declarations()
    -> Result<(), Box<dyn std::error::Error>> {
        let legacy = configuration()?;
        assert_eq!(legacy.tasks, DeliveryTasks::default());
        assert_eq!(legacy.tasks.collect.limit, 20);
        assert!(!legacy.tasks.collect.verify_attestations);
        assert!(legacy.tasks.verify.verify_attestations);
        let partial: DeliveryTasks =
            serde_json::from_value(json!({"collect":{},"verify":{"limit":8}}))?;
        assert_eq!(partial.collect, DeliveryTask::default());
        assert_eq!(partial.verify.limit, 8);
        assert!(partial.verify.verify_attestations);
        let explicit: DeliveryTasks =
            serde_json::from_value(json!({"verify":{"verifyAttestations":false}}))?;
        assert!(!explicit.verify.verify_attestations);
        assert_eq!(explicit.verify.limit, 20);
        assert!(
            serde_json::from_value::<DeliveryTasks>(json!({"verify":{"run":"shell command"}}))
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn delivery_rejects_out_of_bounds_task_limits() -> Result<(), Box<dyn std::error::Error>> {
        for limit in [0, 101] {
            let mut config = configuration()?;
            config.tasks.collect.limit = limit;
            assert!(config.validate().is_err());
            config.tasks.collect.limit = 20;
            config.tasks.verify.limit = limit;
            assert!(config.validate().is_err());
        }
        Ok(())
    }

    fn configuration() -> Result<GitHubDelivery, serde_json::Error> {
        serde_json::from_value(json!({
            "kind":"github", "repository":{"host":"github.com","owner":"example","name":"api"},
            "builds":{"workflow":".github/workflows/build.yml","artifactNames":[]},
            "releases":{"includeDrafts":false,"includePrereleases":false},
            "environments":[{"name":"production", "deployments":{"environment":"production","task":"deploy"},
                "runtime":{"workflow":".github/workflows/probe.yml","artifactName":"runtime","schema":"scryr.runtime.v1","maxAgeSeconds":900},
                "compatibility":{"workflow":".github/workflows/check.yml","artifactName":"compatibility","schema":"scryr.compatibility.v1","maxAgeSeconds":900},
                "rollback":{"requireArtifactAvailable":true,"requireVerifiedProvenance":true,"requireCompatibilityEvidence":true}
            }]
        }))
    }

    fn snapshot(now: DateTime<Utc>) -> Result<DeliverySnapshot, serde_json::Error> {
        let candidate = format!("sha256:{}", "a".repeat(64));
        let current = format!("sha256:{}", "b".repeat(64));
        let old = now - chrono::Duration::minutes(5);
        let mut result = DeliverySnapshot::new("github.com".into(), "example/api".into());
        result.observed_at = now;
        result.configuration = Some(configuration()?);
        result.sources = vec![DeliverySourceState {
            category: DeliverySourceCategory::Artifacts,
            environment: None,
            state: DeliveryEvidenceState::Available,
            message: None,
        }];
        result.artifacts = serde_json::from_value(json!([{
            "id":"asset:1", "kind":"release_asset", "name":"api", "digest":candidate,
            "runId":"10", "releaseId":"1", "expired":false, "url":"https://github.com/example/api/releases/1",
            "provenance":"verified"
        }]))?;
        result.deployments = serde_json::from_value(json!([
            {"id":"1","environment":"production","commitSha":"a".repeat(40),"status":"inactive","createdAt":old,"updatedAt":now,"url":null,
             "statuses":[{"status":"success","createdAt":old,"url":null},{"status":"inactive","createdAt":now,"url":null}]},
            {"id":"2","environment":"production","commitSha":"b".repeat(40),"status":"success","createdAt":now,"updatedAt":now,"url":null}
        ]))?;
        result.links = serde_json::from_value(json!([
            {"schema":"scryr.delivery.v1","manifestId":"api","environment":"production","deploymentId":"1","buildRunId":"10","buildAttempt":1,"sourceRevision":"a".repeat(40),"artifactDigest":candidate,"releaseId":"1","configurationRevision":"old"},
            {"schema":"scryr.delivery.v1","manifestId":"api","environment":"production","deploymentId":"2","buildRunId":"11","buildAttempt":1,"sourceRevision":"b".repeat(40),"artifactDigest":current,"releaseId":"2","configurationRevision":"config"}
        ]))?;
        result.runtime = serde_json::from_value(json!([{
            "schema":"scryr.runtime.v1","manifestId":"api","environment":"production","deploymentId":"2",
            "configurationRevision":"config","schemaRevision":"schema-3","observedAt":now,
            "workloads":[{"name":"api","artifactDigest":current,"replicas":3,"readyReplicas":3}]
        }]))?;
        result.compatibility = serde_json::from_value(json!([{
            "schema":"scryr.compatibility.v1","manifestId":"api","environment":"production","candidateDigest":candidate,
            "currentSchemaRevision":"schema-3","currentConfigurationRevision":"config","candidateConfigurationRevision":"old","suiteRevision":"tests-1","observedAt":now,
            "checks":[{"name":"migration compatibility","outcome":"passed","message":null}]
        }]))?;
        Ok(result)
    }

    #[test]
    fn eligibility_requires_explicit_fresh_matching_evidence()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = configuration()?;
        config.validate()?;
        let now = Utc::now();
        let evidence = snapshot(now)?;
        evidence.validate_for_manifest("api")?;
        let assessment = config.assess(Some(&evidence), now);
        assert_eq!(assessment[0].eligibility, DeliveryEligibility::Eligible);
        assert_eq!(assessment[0].candidate_deployment_id.as_deref(), Some("1"));
        assert_eq!(assessment[0].running_matches_deployment, Some(true));
        let stale = config.assess(Some(&evidence), now + chrono::Duration::minutes(16));
        assert_eq!(stale[0].eligibility, DeliveryEligibility::Unknown);
        assert_eq!(stale[0].runtime_state, DeliveryEvidenceState::Stale);
        Ok(())
    }

    #[test]
    fn absent_runtime_and_mismatched_schema_never_imply_compatibility()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = configuration()?;
        let now = Utc::now();
        let mut evidence = snapshot(now)?;
        evidence.compatibility[0].current_schema_revision = "other-schema".into();
        assert_eq!(
            config.assess(Some(&evidence), now)[0].eligibility,
            DeliveryEligibility::Unknown
        );
        evidence.runtime.clear();
        assert_eq!(
            config.assess(Some(&evidence), now)[0].running_matches_deployment,
            None
        );
        assert_eq!(
            config.assess(None, now)[0].eligibility,
            DeliveryEligibility::Unknown
        );
        Ok(())
    }

    #[test]
    fn mixed_rollout_or_expired_artifact_blocks_candidate() -> Result<(), Box<dyn std::error::Error>>
    {
        let config = configuration()?;
        let now = Utc::now();
        let mut evidence = snapshot(now)?;
        evidence.runtime[0].workloads.push(DeliveryRuntimeWorkload {
            name: "api-old".into(),
            artifact_digest: format!("sha256:{}", "c".repeat(64)),
            replicas: 1,
            ready_replicas: 1,
        });
        assert_eq!(
            config.assess(Some(&evidence), now)[0].eligibility,
            DeliveryEligibility::Blocked
        );
        evidence.runtime[0].workloads.pop();
        evidence.artifacts[0].expired = true;
        assert_eq!(
            config.assess(Some(&evidence), now)[0].eligibility,
            DeliveryEligibility::Blocked
        );
        Ok(())
    }

    #[test]
    fn failed_new_deployment_does_not_replace_observed_running_release()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = configuration()?;
        let now = Utc::now();
        let mut evidence = snapshot(now)?;
        let mut failed = evidence.deployments[1].clone();
        failed.id = "3".into();
        failed.created_at = now + chrono::Duration::seconds(1);
        failed.updated_at = now + chrono::Duration::seconds(2);
        failed.status = DeliveryDeploymentStatus::Failure;
        evidence.deployments.push(failed);
        let unconfirmed = config.assess(Some(&evidence), now + chrono::Duration::seconds(3));
        assert_eq!(unconfirmed[0].running_matches_deployment, None);
        assert_eq!(unconfirmed[0].eligibility, DeliveryEligibility::Unknown);
        evidence.runtime[0].observed_at = now + chrono::Duration::seconds(3);
        let confirmed = config.assess(Some(&evidence), now + chrono::Duration::seconds(3));
        assert_eq!(confirmed[0].running_matches_deployment, Some(true));
        assert_eq!(confirmed[0].eligibility, DeliveryEligibility::Eligible);
        Ok(())
    }

    #[test]
    fn invalid_evidence_is_rejected_at_ingestion() -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let mut evidence = snapshot(now)?;
        assert!(
            evidence
                .validate_for_manifest("different-component")
                .is_err()
        );
        evidence.links[0].deployment_id = "unrelated".into();
        assert!(evidence.validate().is_err());
        let mut evidence = snapshot(now)?;
        evidence.runtime[0].observed_at = now + chrono::Duration::hours(1);
        assert!(evidence.validate().is_err());
        let mut evidence = snapshot(now)?;
        evidence.compatibility[0].checks.clear();
        assert!(evidence.validate().is_err());
        Ok(())
    }

    #[test]
    fn configurations_require_valid_unique_selectors() -> Result<(), Box<dyn std::error::Error>> {
        let mut config = configuration()?;
        config.environments.push(config.environments[0].clone());
        assert!(config.validate().is_err());
        config.environments.clear();
        config.validate()?;
        config.builds.workflow = ".github/workflows/../../unsafe.yml".into();
        assert!(config.validate().is_err());
        Ok(())
    }
    #[test]
    fn compatibility_must_cover_the_configuration_that_would_be_restored()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = configuration()?;
        let now = Utc::now();
        let mut evidence = snapshot(now)?;
        evidence.compatibility[0].candidate_configuration_revision = None;
        assert_eq!(
            config.assess(Some(&evidence), now)[0].eligibility,
            DeliveryEligibility::Unknown
        );
        evidence.links[0].configuration_revision = "config".into();
        assert_eq!(
            config.assess(Some(&evidence), now)[0].eligibility,
            DeliveryEligibility::Eligible
        );
        evidence.compatibility[0].candidate_configuration_revision = Some("other".into());
        assert_eq!(
            config.assess(Some(&evidence), now)[0].eligibility,
            DeliveryEligibility::Unknown
        );
        Ok(())
    }
    #[test]
    fn newer_active_or_unknown_deployments_prevent_rollback_eligibility()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = configuration()?;
        let now = Utc::now();
        for status in [
            DeliveryDeploymentStatus::Pending,
            DeliveryDeploymentStatus::Queued,
            DeliveryDeploymentStatus::InProgress,
            DeliveryDeploymentStatus::Unknown,
        ] {
            let mut evidence = snapshot(now)?;
            let mut newer = evidence.deployments[1].clone();
            newer.id = "3".into();
            newer.created_at = now + chrono::Duration::seconds(1);
            newer.updated_at = newer.created_at;
            newer.status = status;
            evidence.deployments.push(newer);
            evidence.runtime[0].observed_at = now + chrono::Duration::seconds(2);
            let assessment = config.assess(Some(&evidence), now + chrono::Duration::seconds(2));
            assert_eq!(assessment[0].eligibility, DeliveryEligibility::Unknown);
            assert_eq!(assessment[0].running_matches_deployment, None);
        }
        Ok(())
    }

    #[test]
    fn retained_verified_content_is_not_shadowed_by_an_expired_copy()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = configuration()?;
        let now = Utc::now();
        let mut evidence = snapshot(now)?;
        let mut expired = evidence.artifacts[0].clone();
        expired.id = "expired-actions-copy".into();
        expired.kind = DeliveryArtifactKind::ActionsArtifact;
        expired.expired = true;
        expired.provenance = DeliveryVerification::Unknown;
        evidence.artifacts.insert(0, expired);
        let assessment = config.assess(Some(&evidence), now);
        assert_eq!(assessment[0].eligibility, DeliveryEligibility::Eligible);
        evidence.artifacts.reverse();
        assert_eq!(
            config.assess(Some(&evidence), now)[0].eligibility,
            DeliveryEligibility::Eligible
        );
        Ok(())
    }
    #[test]
    fn historical_success_does_not_qualify_a_currently_failed_or_active_candidate()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = configuration()?;
        let now = Utc::now();
        for status in [
            DeliveryDeploymentStatus::Failure,
            DeliveryDeploymentStatus::Error,
            DeliveryDeploymentStatus::InProgress,
            DeliveryDeploymentStatus::Unknown,
        ] {
            let mut evidence = snapshot(now)?;
            evidence.deployments[0].status = status;
            let assessment = config.assess(Some(&evidence), now);
            assert_eq!(assessment[0].candidate_deployment_id, None);
            assert_eq!(assessment[0].eligibility, DeliveryEligibility::Unknown);
        }
        Ok(())
    }
}
