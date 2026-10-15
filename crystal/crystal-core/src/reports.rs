//! Versioned operational observations, independent of manifest generation.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// One immutable, retry-stable operational observation.
pub struct Report {
    /// Wire schema version; currently one.
    pub schema_version: u32,
    /// Collector or artifact format.
    pub source: String,
    /// Independent suite, path or environment identity.
    pub scope: String,
    /// Stable provider run or snapshot identity.
    pub run_id: String,
    /// Run attempt, starting at one.
    pub attempt: u32,
    /// Commit associated with the observation, when known.
    pub commit_sha: Option<String>,
    /// Branch associated with the observation, when known.
    pub branch: Option<String>,
    /// Source completion or snapshot time, independent of delivery time.
    pub observed_at: DateTime<Utc>,
    /// Optional human-readable report location.
    pub report_url: Option<String>,
    /// Typed observed values.
    pub data: ReportData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
/// Validated payloads supported by operational reporting.
pub enum ReportData {
    /// Results from one complete test scope.
    Tests {
        /// Successful test cases.
        passing: u64,
        /// Test cases with assertion failures.
        failing: u64,
        /// Test cases with execution errors.
        errors: u64,
        /// Test cases not executed.
        skipped: u64,
        /// Cumulative testcase duration in seconds.
        duration: f64,
    },
    /// Line coverage for one source scope.
    Coverage {
        /// Executable source lines covered.
        covered: u64,
        /// Total executable source lines.
        total: u64,
    },
    /// Complete dependency security snapshot.
    Dependencies {
        /// Provider owner/repository name.
        repository: String,
        /// Exact dependency manifest path returned by the provider.
        #[serde(alias = "manifest_path")]
        manifest_path: String,
        /// Complete path-scoped security snapshot, including resolved alerts.
        alerts: Vec<DependencyAlert>,
    },
    /// One environment deployment observation.
    Deployment {
        /// Target deployment environment; matches the scope.
        environment: String,
        /// Observed deployment state.
        status: String,
        /// Deployed version or commit identifier.
        version: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A dependency security finding from a complete provider snapshot.
pub struct DependencyAlert {
    /// Repository-scoped provider alert identity.
    pub number: u64,
    /// Affected package name.
    pub package: String,
    /// Package ecosystem.
    pub ecosystem: String,
    /// Exact dependency manifest path returned by the provider.
    pub manifest_path: String,
    /// Provider severity: low, medium, high or critical.
    pub severity: String,
    /// Provider state: open, fixed, dismissed or `auto_dismissed`.
    pub state: String,
    /// Provider alert URL.
    pub url: String,
}
impl Report {
    /// Stable report category used in storage and map cards.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self.data {
            ReportData::Tests { .. } => "tests",
            ReportData::Coverage { .. } => "coverage",
            ReportData::Dependencies { .. } => "dependencies",
            ReportData::Deployment { .. } => "deployment",
        }
    }
    /// Validate the version, identities, counts and provider states.
    /// # Errors
    /// Returns a descriptive error for invalid observations.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.attempt == 0
            || [&self.scope, &self.source, &self.run_id]
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 256)
        {
            return Err(
                "report requires version 1, nonempty source/scope/runId and positive attempt"
                    .into(),
            );
        }
        if self
            .report_url
            .as_ref()
            .is_some_and(|s| !s.starts_with("https://") && !s.starts_with("http://"))
        {
            return Err("invalid report URL".into());
        }
        match &self.data {
            ReportData::Tests {
                duration,
                passing,
                failing,
                errors,
                skipped,
            } => {
                if !duration.is_finite()
                    || *duration < 0.0
                    || passing
                        .checked_add(*failing)
                        .and_then(|n| n.checked_add(*errors))
                        .and_then(|n| n.checked_add(*skipped))
                        .is_none()
                {
                    return Err("invalid test counts or duration".into());
                }
            }
            ReportData::Coverage { covered, total } => {
                if covered > total {
                    return Err("covered must not exceed total".into());
                }
            }
            ReportData::Dependencies {
                repository,
                manifest_path,
                alerts,
            } => {
                if repository.split('/').count() != 2 || manifest_path.is_empty() {
                    return Err("repository and manifest path are required".into());
                }
                let mut ids = std::collections::HashSet::new();
                for a in alerts {
                    if !ids.insert(a.number)
                        || a.package.is_empty()
                        || a.manifest_path != *manifest_path
                        || !matches!(
                            a.state.as_str(),
                            "open" | "fixed" | "dismissed" | "auto_dismissed"
                        )
                        || !matches!(a.severity.as_str(), "low" | "medium" | "high" | "critical")
                    {
                        return Err("invalid or duplicate dependency alert".into());
                    }
                }
            }
            ReportData::Deployment {
                environment,
                status,
                version,
            } => {
                if environment != &self.scope
                    || version.is_empty()
                    || !matches!(
                        status.as_str(),
                        "pending" | "in_progress" | "success" | "failure" | "inactive"
                    )
                {
                    return Err(
                        "deployment requires matching environment scope, version and valid status"
                            .into(),
                    );
                }
            }
        }
        Ok(())
    }
    /// Project values onto existing manifest summary fields.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Percentages are approximate display values.
    pub fn summary(&self) -> Value {
        match &self.data {
            ReportData::Tests {
                passing,
                failing,
                errors,
                skipped,
                duration,
            } => {
                json!({"total":passing+failing+errors+skipped,"passing":passing,"failing":failing,"errors":errors,"skipped":skipped,"executionTime":duration,"lastRun":self.observed_at})
            }
            ReportData::Coverage { covered, total } => {
                json!({"coverage":if *total == 0 {None} else {Some(*covered as f64 / *total as f64 * 100.0)}})
            }
            ReportData::Dependencies { alerts, .. } => {
                let open: Vec<_> = alerts.iter().filter(|a| a.state == "open").collect();
                let packages: std::collections::HashSet<_> = open
                    .iter()
                    .map(|a| (&a.ecosystem, &a.package, &a.manifest_path))
                    .collect();
                let severity = open
                    .iter()
                    .map(|a| a.severity.as_str())
                    .max_by_key(|s| match *s {
                        "critical" => 4,
                        "high" => 3,
                        "medium" => 2,
                        _ => 1,
                    });
                json!({"vulnerableDeps":packages.len(),"openAlerts":open.len(),"maxSeverity":severity})
            }
            ReportData::Deployment { .. } => json!({}),
        }
    }
}

/// Validate an explicit identity that is stable across source edits.
/// # Errors
/// Rejects empty, oversized or malformed identifiers.
pub fn validate_manifest_id(manifest_id: &str) -> Result<(), String> {
    if manifest_id.is_empty()
        || manifest_id.len() > 256
        || !manifest_id.starts_with(|c: char| c.is_ascii_alphanumeric())
        || !manifest_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:/-".contains(c))
    {
        return Err("manifestId must be a stable identifier of 1 to 256 characters".into());
    }
    Ok(())
}
