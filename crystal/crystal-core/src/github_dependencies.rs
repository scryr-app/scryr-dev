//! Canonical, complete GitHub dependency inventory and open-alert snapshots.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Independently collected dependency components for one repository.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubDependencySnapshot {
    /// Canonical GitHub owner/repository identifier.
    pub repository: String,
    /// Latest complete inventory; absent means this component was not collected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory: Option<DependencyInventory>,
    /// Latest complete open-alert set; absent means this component was not collected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<DependencySecurity>,
}

/// Complete dependency inventory at one collection time.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DependencyInventory {
    /// Provider collection time, separate from receipt time.
    pub observed_at: DateTime<Utc>,
    /// All packages in the collected inventory.
    pub packages: Vec<DependencyPackage>,
    /// Direct dependency count, when relationships establish it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_deps: Option<u64>,
    /// Transitive dependency count, when relationships establish it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transitive_deps: Option<u64>,
}

/// One package from a complete inventory.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DependencyPackage {
    /// Unique provider package identity within this snapshot.
    pub id: String,
    /// Package display name.
    pub name: String,
    /// Installed version, when supplied.
    pub version: Option<String>,
    /// Declared or concluded license, when supplied.
    pub license: Option<String>,
    /// Package URL identity, when supplied.
    pub purl: Option<String>,
}

/// Complete set of open Dependabot alerts at one collection time.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DependencySecurity {
    /// Provider collection time, separate from receipt time.
    pub observed_at: DateTime<Utc>,
    /// Open alerts only; an empty array explicitly clears prior alerts.
    pub alerts: Vec<GithubDependencyAlert>,
}

/// An open repository-scoped GitHub dependency alert.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubDependencyAlert {
    /// Positive repository-scoped alert number.
    pub number: u64,
    /// Vulnerable package name.
    pub package: String,
    /// Package ecosystem supplied by GitHub.
    pub ecosystem: String,
    /// Repository-relative dependency manifest path.
    pub manifest_path: String,
    /// Normalized severity: low, medium, high, or critical.
    pub severity: String,
    /// Exact GitHub repository Dependabot alert page.
    pub url: String,
    /// GitHub advisory identity, when available.
    pub ghsa_id: Option<String>,
    /// CVE identity, when available.
    pub cve_id: Option<String>,
    /// Advisory summary, when available.
    pub summary: Option<String>,
    /// Vulnerable package version range, when available.
    pub vulnerable_version_range: Option<String>,
    /// First fixed version, when available.
    pub first_patched_version: Option<String>,
}

impl GithubDependencySnapshot {
    /// Validate complete component snapshots before accepting them into storage.
    ///
    /// # Errors
    /// Rejects malformed identities, duplicates, unsupported severities, unsafe alert
    /// URLs, empty snapshots, or snapshots exceeding collection/payload bounds.
    pub fn validate(&self) -> Result<(), String> {
        if !valid_repository(&self.repository) {
            return Err("GitHub dependency snapshot requires a canonical owner/repository".into());
        }
        if self.inventory.is_none() && self.security.is_none() {
            return Err("GitHub dependency snapshot requires inventory or security".into());
        }
        if let Some(inventory) = &self.inventory {
            let mut ids = HashSet::new();
            if inventory.packages.len() > 10_000
                || inventory.direct_deps.is_some_and(|n| n > 10_000)
                || inventory.transitive_deps.is_some_and(|n| n > 10_000)
                || inventory.packages.iter().any(|package| {
                    !valid_identity(&package.id)
                        || !valid_identity(&package.name)
                        || !ids.insert(&package.id)
                })
            {
                return Err("invalid or oversized GitHub dependency inventory".into());
            }
        }
        if let Some(security) = &self.security {
            let mut numbers = HashSet::new();
            if security.alerts.len() > 10_000
                || security.alerts.iter().any(|alert| {
                    alert.number == 0
                        || !numbers.insert(alert.number)
                        || !valid_identity(&alert.package)
                        || !valid_identity(&alert.ecosystem)
                        || !valid_manifest_path(&alert.manifest_path)
                        || !matches!(
                            alert.severity.as_str(),
                            "low" | "medium" | "high" | "critical"
                        )
                        || !valid_alert_url(&self.repository, alert)
                })
            {
                return Err("invalid or oversized GitHub dependency security snapshot".into());
            }
        }
        if serde_json::to_vec(self)
            .map_err(|error| error.to_string())?
            .len()
            > 2_000_000
        {
            return Err("GitHub dependency snapshot exceeds 2 MB".into());
        }
        Ok(())
    }
}

fn valid_repository(repository: &str) -> bool {
    let parts: Vec<_> = repository.split('/').collect();
    parts.len() == 2
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.len() <= 100
                && *part != "."
                && *part != ".."
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
        })
}

fn valid_identity(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control)
}

fn valid_manifest_path(value: &str) -> bool {
    valid_identity(value)
        && !value.contains('\\')
        && value
            .split('/')
            .all(|part| !matches!(part, "" | "." | ".."))
}

fn valid_alert_url(repository: &str, alert: &GithubDependencyAlert) -> bool {
    alert
        .url
        .strip_prefix("https://github.com/")
        .and_then(|path| path.split_once("/security/dependabot/"))
        .is_some_and(|(repo, number)| {
            repo.eq_ignore_ascii_case(repository) && number == alert.number.to_string()
        })
}
