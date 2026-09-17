//! Canonical typed local evidence; declarations and observations never execute code.
#![allow(missing_docs)]
use async_graphql::{Enum, SimpleObject, Union};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Enum,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSection {
    Repository,
    Checks,
    Metrics,
    Tests,
    Dependencies,
    Performance,
}
impl EvidenceSection {
    pub const ALL: [Self; 6] = [
        Self::Repository,
        Self::Checks,
        Self::Metrics,
        Self::Tests,
        Self::Dependencies,
        Self::Performance,
    ];
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Repository => "repository",
            Self::Checks => "checks",
            Self::Metrics => "metrics",
            Self::Tests => "tests",
            Self::Dependencies => "dependencies",
            Self::Performance => "performance",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Enum)]
#[serde(rename_all = "snake_case")]
pub enum CollectorState {
    Waiting,
    Running,
    Ready,
    MissingTool,
    IncompatibleTool,
    NeedsLogin,
    Error,
    Cancelled,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceObservation {
    pub schema_version: u32,
    pub observation_id: String,
    pub manifest_id: String,
    pub section: EvidenceSection,
    pub collector_id: String,
    pub integration: String,
    pub workspace_id: String,
    #[serde(default = "local_environment")]
    pub environment: String,
    pub scope: String,
    pub plan_revision: String,
    pub collector_revision: String,
    pub run_id: String,
    pub attempt: u32,
    pub observed_at: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub input_fingerprint: String,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub dirty: Option<bool>,
    pub tool_version: Option<String>,
    pub upstream_fingerprint: Option<String>,
    pub policy_revision: Option<String>,
    pub result: EvidenceResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectorStatus {
    pub manifest_id: String,
    pub section: EvidenceSection,
    pub collector_id: String,
    pub workspace_id: String,
    pub integration: String,
    pub collector_revision: String,
    pub state: CollectorState,
    pub message: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub freshness_seconds: u64,
    pub input_fingerprint: Option<String>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectorEvidence {
    pub manifest_id: String,
    pub section: EvidenceSection,
    pub collector_id: String,
    pub integration: String,
    pub workspace_id: String,
    pub state: CollectorState,
    pub message: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub freshness_seconds: u64,
    pub stale: bool,
    pub outdated: bool,
    pub latest: Option<EvidenceObservation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitResult {
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub dirty: bool,
    pub changed_files: u64,
    pub ahead: u64,
    pub behind: u64,
    pub remote_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub state: String,
    pub review_decision: Option<String>,
    pub head_ref_name: Option<String>,
    pub head_ref_oid: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PullRequestsResult {
    pub repository: String,
    pub items: Vec<PullRequest>,
    pub complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowRun {
    pub run_id: String,
    pub attempt: u32,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub branch: String,
    pub commit: String,
    pub url: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowsResult {
    pub repository: String,
    pub items: Vec<WorkflowRun>,
    pub complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Diagnostic {
    pub message: String,
    pub path: Option<String>,
    pub line: Option<u64>,
    pub severity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub duration_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestCase {
    pub name: String,
    pub suite: String,
    pub status: String,
    pub message: Option<String>,
    pub duration_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestResult {
    pub suite: String,
    pub passing: u64,
    pub failing: u64,
    pub errors: u64,
    pub skipped: u64,
    pub duration_seconds: f64,
    pub cases: Vec<TestCase>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoverageResult {
    pub suite: String,
    pub covered: u64,
    pub total: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    pub purl: Option<String>,
    pub paths: Vec<String>,
    pub licenses: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryResult {
    pub packages: Vec<Package>,
    pub relationships: Vec<DependencyEdge>,
    pub complete: bool,
    pub artifact_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LicenseFinding {
    pub package_id: String,
    pub expression: Option<String>,
    pub decision: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LicenseResult {
    pub items: Vec<LicenseFinding>,
    pub policy_revision: String,
    pub inventory_hash: String,
    pub complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VulnerabilityFinding {
    pub package_id: String,
    pub advisory_id: String,
    pub aliases: Vec<String>,
    pub severity: String,
    pub fix_versions: Vec<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VulnerabilityResult {
    pub items: Vec<VulnerabilityFinding>,
    pub inventory_hash: String,
    pub database_age_seconds: Option<u64>,
    pub database_version: Option<String>,
    pub complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetricLabel {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetricSample {
    pub name: String,
    pub title: Option<String>,
    pub labels: Vec<MetricLabel>,
    pub value: f64,
    pub unit: Option<String>,
    pub metric_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetricsResult {
    pub samples: Vec<MetricSample>,
    pub scraped_at: DateTime<Utc>,
    pub complete: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BenchmarkResult {
    pub name: String,
    pub mean_seconds: f64,
    pub stddev_seconds: f64,
    pub median_seconds: f64,
    pub runs: u32,
    pub command: String,
    pub baseline_mean_seconds: Option<f64>,
    pub machine: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Union)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum EvidenceResult {
    Git(GitResult),
    PullRequests(PullRequestsResult),
    Workflows(WorkflowsResult),
    Check(CheckResult),
    Test(TestResult),
    Coverage(CoverageResult),
    Inventory(InventoryResult),
    License(LicenseResult),
    Vulnerability(VulnerabilityResult),
    Metrics(MetricsResult),
    Benchmark(BenchmarkResult),
}

fn local_environment() -> String {
    "local".into()
}
/// Bounded, stable identity used for manifests, collectors and workspaces.
pub fn validate_identity(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 256
        || !value.starts_with(|c: char| c.is_ascii_alphanumeric())
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:/-".contains(c))
    {
        return Err("identity must contain 1..256 ASCII letters, digits, or ._:/- and start with a letter or digit".into());
    }
    Ok(())
}
impl EvidenceObservation {
    /// Reject malformed or mismatched evidence before it reaches storage.
    pub fn validate(&self) -> Result<(), String> {
        for id in [
            &self.observation_id,
            &self.manifest_id,
            &self.collector_id,
            &self.workspace_id,
            &self.run_id,
        ] {
            validate_identity(id)?;
        }
        if self.schema_version != 1
            || self.attempt == 0
            || self.observed_at < self.started_at
            || [
                self.scope.as_str(),
                self.plan_revision.as_str(),
                self.collector_revision.as_str(),
                self.input_fingerprint.as_str(),
                self.environment.as_str(),
            ]
            .iter()
            .any(|s| s.is_empty() || s.len() > 1024)
        {
            return Err("evidence requires schemaVersion 1, positive attempt, ordered times and nonempty provenance".into());
        }
        if self.section != self.result.section() || !self.result.accepts(&self.integration) {
            return Err("result does not match collector integration and section".into());
        }
        self.result.validate()
    }
}
impl CollectorStatus {
    pub fn validate(&self) -> Result<(), String> {
        for id in [&self.manifest_id, &self.collector_id, &self.workspace_id] {
            validate_identity(id)?;
        }
        if self.collector_revision.is_empty()
            || self.freshness_seconds == 0
            || self.message.as_ref().is_some_and(|s| s.len() > 4096)
            || crate::collectors::integration_section(&self.integration) != Some(self.section)
        {
            return Err("invalid collector status".into());
        }
        Ok(())
    }
}
impl EvidenceResult {
    #[must_use]
    pub const fn section(&self) -> EvidenceSection {
        match self {
            Self::Git(_) | Self::PullRequests(_) | Self::Workflows(_) => {
                EvidenceSection::Repository
            }
            Self::Check(_) => EvidenceSection::Checks,
            Self::Test(_) | Self::Coverage(_) => EvidenceSection::Tests,
            Self::Inventory(_) | Self::License(_) | Self::Vulnerability(_) => {
                EvidenceSection::Dependencies
            }
            Self::Metrics(_) => EvidenceSection::Metrics,
            Self::Benchmark(_) => EvidenceSection::Performance,
        }
    }
    #[must_use]
    pub fn accepts(&self, integration: &str) -> bool {
        match self {
            Self::Git(_) => integration == "git_status",
            Self::PullRequests(_) => integration == "github_pull_requests",
            Self::Workflows(_) => integration == "github_actions",
            Self::Check(_) => matches!(integration, "ruff" | "biome" | "clippy" | "mise_task"),
            Self::Test(_) => matches!(integration, "pytest" | "vitest" | "nextest" | "junit"),
            Self::Coverage(_) => matches!(integration, "lcov" | "cobertura"),
            Self::Inventory(_) => integration == "syft_inventory",
            Self::License(_) => integration == "grant_license",
            Self::Vulnerability(_) => integration == "grype_scan",
            Self::Metrics(_) => matches!(integration, "openmetrics" | "docker_stats"),
            Self::Benchmark(_) => integration == "hyperfine",
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let duration = |n: f64| n.is_finite() && n >= 0.0;
        let valid = match self {
            Self::Check(r) => duration(r.duration_seconds),
            Self::Test(r) => {
                duration(r.duration_seconds)
                    && r.cases.iter().all(|c| duration(c.duration_seconds))
                    && r.passing
                        .checked_add(r.failing)
                        .and_then(|n| n.checked_add(r.errors))
                        .and_then(|n| n.checked_add(r.skipped))
                        .is_some()
            }
            Self::Coverage(r) => r.covered <= r.total,
            Self::Metrics(r) => {
                r.samples.len() <= 10000
                    && r.samples
                        .iter()
                        .all(|s| !s.name.is_empty() && s.value.is_finite())
            }
            Self::Benchmark(r) => {
                r.runs > 0
                    && [r.mean_seconds, r.stddev_seconds, r.median_seconds]
                        .into_iter()
                        .all(duration)
                    && r.baseline_mean_seconds.is_none_or(duration)
            }
            Self::Inventory(r) => {
                let ids: std::collections::HashSet<_> =
                    r.packages.iter().map(|p| p.id.as_str()).collect();
                !r.artifact_hash.is_empty()
                    && ids.len() == r.packages.len()
                    && r.packages
                        .iter()
                        .all(|p| !p.id.is_empty() && !p.name.is_empty())
                    && r.relationships
                        .iter()
                        .all(|e| ids.contains(e.from.as_str()) && ids.contains(e.to.as_str()))
            }
            Self::License(r) => {
                !r.inventory_hash.is_empty()
                    && !r.policy_revision.is_empty()
                    && r.items
                        .iter()
                        .all(|i| matches!(i.decision.as_str(), "allow" | "deny" | "review"))
            }
            Self::Vulnerability(r) => {
                !r.inventory_hash.is_empty()
                    && r.items
                        .iter()
                        .all(|i| !i.package_id.is_empty() && !i.advisory_id.is_empty())
            }
            Self::Workflows(r) => r.items.iter().all(|i| i.attempt > 0),
            Self::Git(_) | Self::PullRequests(_) => true,
        };
        if valid {
            Ok(())
        } else {
            Err("invalid evidence result values".into())
        }
    }
}
