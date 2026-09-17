//! Typed, inert collector declarations shared by the SDK, server and local runner.
#![allow(missing_docs)]
use crate::evidence::{EvidenceSection, validate_identity};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvRef {
    pub name: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolRequirement {
    pub version: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Schedule {
    pub startup: bool,
    pub every: Option<f64>,
    pub watch: Vec<String>,
    pub upstream_changed: bool,
    pub manual: bool,
    pub debounce: f64,
}
impl Default for Schedule {
    fn default() -> Self {
        Self {
            startup: false,
            every: None,
            watch: vec![],
            upstream_changed: false,
            manual: true,
            debounce: 1.0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectorCommon {
    pub id: String,
    pub schedule: Schedule,
    pub timeout: f64,
    pub freshness: f64,
    pub env: BTreeMap<String, EnvRef>,
    pub tool: Option<ToolRequirement>,
}
impl Default for CollectorCommon {
    fn default() -> Self {
        Self {
            id: String::new(),
            schedule: Schedule::default(),
            timeout: 300.0,
            freshness: 3600.0,
            env: BTreeMap::new(),
            tool: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SbomRef {
    pub collector_id: String,
    pub manifest_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LicensePolicy {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub unknown: String,
}
impl Default for LicensePolicy {
    fn default() -> Self {
        Self {
            allow: vec![],
            deny: vec![],
            unknown: "review".into(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "dot")]
    pub cwd: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MetricSelection {
    CounterRate {
        metric: String,
        title: Option<String>,
    },
    Gauge {
        metric: String,
        title: Option<String>,
    },
    HistogramPercentile {
        metric: String,
        title: Option<String>,
        percentile: f64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitStatusConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GithubPullRequestsConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub repository: String,
    #[serde(default = "twenty")]
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GithubActionsConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub repository: String,
    pub branch: Option<String>,
    pub workflow: Option<String>,
    #[serde(default = "twenty")]
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuffConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    #[serde(default = "dot_paths")]
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BiomeConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    #[serde(default = "dot_paths")]
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClippyConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    #[serde(default = "yes")]
    pub all_targets: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiseTaskConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    pub task: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub forge: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenmetricsConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub endpoint: String,
    #[serde(default)]
    pub series: Vec<MetricSelection>,
    #[serde(default = "two_hundred")]
    pub max_series: u32,
    pub auth: Option<EnvRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DockerStatsConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub containers: Vec<String>,
    #[serde(default = "default_name")]
    pub context: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PytestConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    #[serde(default = "test_paths")]
    pub paths: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VitestConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NextestConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JunitConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub files: Vec<String>,
    #[serde(default = "default_name")]
    pub suite: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LcovConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub files: Vec<String>,
    #[serde(default = "default_name")]
    pub suite: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoberturaConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub files: Vec<String>,
    #[serde(default = "default_name")]
    pub suite: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyftInventoryConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    #[serde(default = "dot")]
    pub directory: String,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrypeScanConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub sbom: Option<SbomRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantLicenseConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub sbom: Option<SbomRef>,
    #[serde(default)]
    pub policy: LicensePolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HyperfineConfig {
    #[serde(flatten)]
    pub common: CollectorCommon,
    pub command: Command,
    #[serde(default = "two")]
    pub warmup: u32,
    #[serde(default = "ten")]
    pub runs: u32,
    pub baseline: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollectorConfig {
    #[serde(rename = "git_status")]
    GitStatus(GitStatusConfig),
    #[serde(rename = "github_pull_requests")]
    GithubPullRequests(GithubPullRequestsConfig),
    #[serde(rename = "github_actions")]
    GithubActions(GithubActionsConfig),
    #[serde(rename = "ruff")]
    Ruff(RuffConfig),
    #[serde(rename = "biome")]
    Biome(BiomeConfig),
    #[serde(rename = "clippy")]
    Clippy(ClippyConfig),
    #[serde(rename = "mise_task")]
    MiseTask(MiseTaskConfig),
    #[serde(rename = "openmetrics")]
    Openmetrics(OpenmetricsConfig),
    #[serde(rename = "docker_stats")]
    DockerStats(DockerStatsConfig),
    #[serde(rename = "pytest")]
    Pytest(PytestConfig),
    #[serde(rename = "vitest")]
    Vitest(VitestConfig),
    #[serde(rename = "nextest")]
    Nextest(NextestConfig),
    #[serde(rename = "junit")]
    Junit(JunitConfig),
    #[serde(rename = "lcov")]
    Lcov(LcovConfig),
    #[serde(rename = "cobertura")]
    Cobertura(CoberturaConfig),
    #[serde(rename = "syft_inventory")]
    SyftInventory(SyftInventoryConfig),
    #[serde(rename = "grype_scan")]
    GrypeScan(GrypeScanConfig),
    #[serde(rename = "grant_license")]
    GrantLicense(GrantLicenseConfig),
    #[serde(rename = "hyperfine")]
    Hyperfine(HyperfineConfig),
}
impl CollectorConfig {
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::GitStatus(_) => "git_status",
            Self::GithubPullRequests(_) => "github_pull_requests",
            Self::GithubActions(_) => "github_actions",
            Self::Ruff(_) => "ruff",
            Self::Biome(_) => "biome",
            Self::Clippy(_) => "clippy",
            Self::MiseTask(_) => "mise_task",
            Self::Openmetrics(_) => "openmetrics",
            Self::DockerStats(_) => "docker_stats",
            Self::Pytest(_) => "pytest",
            Self::Vitest(_) => "vitest",
            Self::Nextest(_) => "nextest",
            Self::Junit(_) => "junit",
            Self::Lcov(_) => "lcov",
            Self::Cobertura(_) => "cobertura",
            Self::SyftInventory(_) => "syft_inventory",
            Self::GrypeScan(_) => "grype_scan",
            Self::GrantLicense(_) => "grant_license",
            Self::Hyperfine(_) => "hyperfine",
        }
    }
    #[must_use]
    pub const fn common(&self) -> &CollectorCommon {
        match self {
            Self::GitStatus(c) => &c.common,
            Self::GithubPullRequests(c) => &c.common,
            Self::GithubActions(c) => &c.common,
            Self::Ruff(c) => &c.common,
            Self::Biome(c) => &c.common,
            Self::Clippy(c) => &c.common,
            Self::MiseTask(c) => &c.common,
            Self::Openmetrics(c) => &c.common,
            Self::DockerStats(c) => &c.common,
            Self::Pytest(c) => &c.common,
            Self::Vitest(c) => &c.common,
            Self::Nextest(c) => &c.common,
            Self::Junit(c) => &c.common,
            Self::Lcov(c) => &c.common,
            Self::Cobertura(c) => &c.common,
            Self::SyftInventory(c) => &c.common,
            Self::GrypeScan(c) => &c.common,
            Self::GrantLicense(c) => &c.common,
            Self::Hyperfine(c) => &c.common,
        }
    }
    #[must_use]
    pub fn id(&self) -> &str {
        if self.common().id.is_empty() {
            self.kind()
        } else {
            &self.common().id
        }
    }
    #[must_use]
    pub fn section(&self) -> EvidenceSection {
        integration_section(self.kind()).unwrap_or(EvidenceSection::Checks)
    }
    #[must_use]
    pub const fn schedule(&self) -> &Schedule {
        &self.common().schedule
    }
    #[must_use]
    pub const fn timeout(&self) -> f64 {
        self.common().timeout
    }
    #[must_use]
    pub fn directory(&self) -> &str {
        match self {
            Self::GitStatus(c) => &c.directory,
            Self::Ruff(c) => &c.directory,
            Self::Biome(c) => &c.directory,
            Self::Clippy(c) => &c.directory,
            Self::MiseTask(c) => &c.directory,
            Self::Pytest(c) => &c.directory,
            Self::Vitest(c) => &c.directory,
            Self::Nextest(c) => &c.directory,
            Self::SyftInventory(c) => &c.directory,
            Self::Hyperfine(c) => &c.command.cwd,
            _ => ".",
        }
    }
    /// # Errors
    /// Returns a serialization error if the configuration cannot be encoded.
    pub fn revision(&self) -> Result<String, String> {
        fingerprint(self)
    }
    #[must_use]
    pub const fn sbom(&self) -> Option<&SbomRef> {
        match self {
            Self::GrypeScan(c) => c.sbom.as_ref(),
            Self::GrantLicense(c) => c.sbom.as_ref(),
            _ => None,
        }
    }
    /// # Errors
    /// Rejects invalid triggers, paths, identities and integration options.
    pub fn validate(&self) -> Result<(), String> {
        validate_identity(self.id())?;
        let c = self.common();
        let s = &c.schedule;
        if !c.timeout.is_finite()
            || !(0.1..=86400.0).contains(&c.timeout)
            || !c.freshness.is_finite()
            || !(1.0..=31_536_000.0).contains(&c.freshness)
            || !s.debounce.is_finite()
            || !(0.0..=3600.0).contains(&s.debounce)
            || s.every
                .is_some_and(|v| !v.is_finite() || !(1.0..=31_536_000.0).contains(&v))
            || !(s.startup
                || s.every.is_some()
                || !s.watch.is_empty()
                || s.upstream_changed
                || s.manual)
        {
            return Err("collector requires bounded durations and at least one trigger".into());
        }
        for (key, value) in &c.env {
            if !env_name(key) || !env_name(&value.name) {
                return Err("environment references must use variable names".into());
            }
        }
        validate_path(self.directory())?;
        for path in &s.watch {
            validate_path(path)?;
        }
        match self {
            Self::GithubPullRequests(v) => repository(&v.repository, v.limit)?,
            Self::GithubActions(v) => repository(&v.repository, v.limit)?,
            Self::MiseTask(v) if v.task.trim().is_empty() || v.task.starts_with('-') => {
                return Err("mise_task requires a task name".into());
            }
            Self::Openmetrics(v) => {
                if !(v.endpoint.starts_with("http://") || v.endpoint.starts_with("https://"))
                    || v.endpoint.contains('@')
                    || !(1..=10000).contains(&v.max_series)
                    || v.auth.as_ref().is_some_and(|r| !env_name(&r.name))
                {
                    return Err("invalid OpenMetrics endpoint or series limit".into());
                }
                for selection in &v.series {
                    let (metric, p) = match selection {
                        MetricSelection::CounterRate { metric, .. }
                        | MetricSelection::Gauge { metric, .. } => (metric, None),
                        MetricSelection::HistogramPercentile {
                            metric, percentile, ..
                        } => (metric, Some(*percentile)),
                    };
                    if metric.is_empty()
                        || p.is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
                    {
                        return Err("invalid metric selection".into());
                    }
                }
            }
            Self::Junit(v) => files(&v.files)?,
            Self::Lcov(v) => files(&v.files)?,
            Self::Cobertura(v) => files(&v.files)?,
            Self::GrantLicense(v) if !matches!(v.policy.unknown.as_str(), "review" | "deny") => {
                return Err("unknown license decision must be review or deny".into());
            }
            Self::Hyperfine(v)
                if v.command.executable.trim().is_empty()
                    || v.runs == 0
                    || v.runs > 10000
                    || v.warmup > 1000 =>
            {
                return Err("invalid benchmark command or run count".into());
            }
            _ => {}
        }
        if let Some(reference) = self.sbom() {
            validate_identity(&reference.collector_id)?;
            if let Some(id) = &reference.manifest_id {
                validate_identity(id)?;
            }
        }
        Ok(())
    }
}
fn env_name(s: &str) -> bool {
    !s.is_empty()
        && s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn validate_path(s: &str) -> Result<(), String> {
    if s.is_empty()
        || std::path::Path::new(s).is_absolute()
        || std::path::Path::new(s)
            .components()
            .any(|p| matches!(p, std::path::Component::ParentDir))
    {
        Err("collector paths must stay relative to the selected project".into())
    } else {
        Ok(())
    }
}
fn files(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() {
        return Err("report collector requires files".into());
    }
    for p in paths {
        validate_path(p)?;
    }
    Ok(())
}
fn repository(name: &str, limit: u32) -> Result<(), String> {
    if name.split('/').count() != 2
        || name.split('/').any(|s| {
            s.is_empty()
                || !s
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        })
        || !(1..=100).contains(&limit)
    {
        Err("repository must be owner/name and limit 1..100".into())
    } else {
        Ok(())
    }
}
#[must_use]
pub fn integration_section(kind: &str) -> Option<EvidenceSection> {
    Some(match kind {
        "git_status" | "github_pull_requests" | "github_actions" => EvidenceSection::Repository,
        "ruff" | "biome" | "clippy" | "mise_task" => EvidenceSection::Checks,
        "openmetrics" | "docker_stats" => EvidenceSection::Metrics,
        "pytest" | "vitest" | "nextest" | "junit" | "lcov" | "cobertura" => EvidenceSection::Tests,
        "syft_inventory" | "grype_scan" | "grant_license" => EvidenceSection::Dependencies,
        "hyperfine" => EvidenceSection::Performance,
        _ => return None,
    })
}
#[derive(Clone, Debug)]
pub struct CollectorDeclaration {
    pub manifest_id: String,
    pub section: EvidenceSection,
    pub config: CollectorConfig,
    pub revision: String,
}
/// Decode and validate only inert declarations; this function never starts a process.
/// # Errors
/// Rejects invalid collectors, duplicate identities and missing or ambiguous inventory references.
pub fn declarations(envelope: &Value) -> Result<Vec<CollectorDeclaration>, String> {
    parse_declarations(envelope, true)
}
/// Decode an already scoped artifact, whose referenced inventory may be outside this diagram.
/// # Errors
/// Rejects invalid collectors or duplicate identities in the scoped artifact.
pub fn declarations_for_projection(envelope: &Value) -> Result<Vec<CollectorDeclaration>, String> {
    parse_declarations(envelope, false)
}
fn parse_declarations(
    envelope: &Value,
    validate_references: bool,
) -> Result<Vec<CollectorDeclaration>, String> {
    let manifests = envelope
        .as_array()
        .or_else(|| envelope.get("manifests").and_then(Value::as_array))
        .ok_or("expected manifests array")?;
    let mut out = Vec::new();
    let mut keys = std::collections::BTreeMap::new();
    for manifest in manifests {
        let mut local_keys = BTreeSet::new();
        for removed in ["github", "cicd", "analytics"] {
            if manifest.get(removed).is_some_and(|v| !v.is_null()) {
                return Err(format!(
                    "{removed} is no longer supported; use typed evidence sections"
                ));
            }
        }
        for section in EvidenceSection::ALL {
            let Some(value) = manifest.get(section.as_str()).filter(|v| !v.is_null()) else {
                continue;
            };
            let list = value
                .as_array()
                .ok_or_else(|| format!("{} must be a typed collector list", section.as_str()))?;
            if list.is_empty() {
                continue;
            }
            let id = manifest["manifestId"]
                .as_str()
                .ok_or("evidence requires a stable manifest_id")?;
            validate_identity(id)?;
            for value in list {
                let config: CollectorConfig = serde_json::from_value(value.clone())
                    .map_err(|e| format!("invalid {} collector: {e}", section.as_str()))?;
                config.validate()?;
                if config.section() != section {
                    return Err(format!(
                        "{} is not valid in {}",
                        config.kind(),
                        section.as_str()
                    ));
                }
                let key = (id.to_owned(), section.as_str(), config.id().to_owned());
                let revision = config.revision()?;
                if !local_keys.insert(key.clone()) {
                    return Err(format!("duplicate collector id {} on {id}", config.id()));
                }
                if let Some(previous) = keys.insert(key, revision.clone()) {
                    // A manifest may appear in several stored diagrams. Passive
                    // reads share one projection for identical declarations.
                    if !validate_references && previous == revision {
                        continue;
                    }
                    return Err(format!("duplicate collector id {} on {id}", config.id()));
                }
                out.push(CollectorDeclaration {
                    manifest_id: id.into(),
                    section,
                    revision,
                    config,
                });
            }
        }
    }
    if !validate_references {
        return Ok(out);
    }
    for declaration in &out {
        if matches!(
            &declaration.config,
            CollectorConfig::GrypeScan(_) | CollectorConfig::GrantLicense(_)
        ) {
            let candidates = out
                .iter()
                .filter(|d| {
                    matches!(d.config, CollectorConfig::SyftInventory(_))
                        && declaration.config.sbom().map_or_else(
                            || d.manifest_id == declaration.manifest_id,
                            |r| {
                                d.manifest_id
                                    == *r.manifest_id.as_ref().unwrap_or(&declaration.manifest_id)
                                    && d.config.id() == r.collector_id
                            },
                        )
                })
                .count();
            if candidates != 1 {
                return Err(format!(
                    "{} requires one unambiguous syft_inventory reference",
                    declaration.config.id()
                ));
            }
        }
    }
    Ok(out)
}
/// Canonical revision of a serializable value, independent of object key order.
/// # Errors
/// Returns serialization errors.
pub fn fingerprint<T: Serialize>(value: &T) -> Result<String, String> {
    use std::fmt::Write as _;
    let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    Ok(Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        }))
}
fn dot() -> String {
    ".".into()
}
fn default_name() -> String {
    "default".into()
}
fn dot_paths() -> Vec<String> {
    vec![dot()]
}
fn test_paths() -> Vec<String> {
    vec!["tests".into()]
}
const fn twenty() -> u32 {
    20
}
const fn two_hundred() -> u32 {
    200
}
const fn yes() -> bool {
    true
}
const fn two() -> u32 {
    2
}
const fn ten() -> u32 {
    10
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn shared_diagram_manifests_use_one_passive_projection() -> Result<(), String> {
        let manifest = json!({"manifestId":"api","repository":[{"kind":"git_status"}]});
        let diagrams = json!([manifest.clone(), manifest]);
        assert_eq!(declarations_for_projection(&diagrams)?.len(), 1);
        assert!(declarations(&diagrams).is_err());
        assert!(declarations_for_projection(&json!([{"manifestId":"api","repository":[{"kind":"git_status"},{"kind":"git_status"}]}])).is_err());
        Ok(())
    }
    #[test]
    fn configs_are_typed_strict_and_round_trip() -> Result<(), String> {
        let value = json!({"kind":"git_status","id":"git","schedule":{"startup":true,"every":15},"directory":"."});
        let config: CollectorConfig = serde_json::from_value(value).map_err(|e| e.to_string())?;
        config.validate()?;
        let roundtrip: CollectorConfig =
            serde_json::from_value(serde_json::to_value(&config).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        assert_eq!(config.revision()?, roundtrip.revision()?);
        assert_eq!(config.section(), EvidenceSection::Repository);
        assert!(
            serde_json::from_value::<CollectorConfig>(
                json!({"kind":"git_status","secret":"password"})
            )
            .is_err()
        );
        assert!(serde_json::from_value::<CollectorConfig>(json!({"kind":"unknown"})).is_err());
        Ok(())
    }
    #[test]
    fn declarations_reject_wrong_sections_duplicates_and_ambiguous_refs() {
        assert!(
            declarations(&json!([{"manifestId":"api","checks":[{"kind":"git_status"}]}])).is_err()
        );
        assert!(declarations(&json!([{"manifestId":"api","repository":[{"kind":"git_status"},{"kind":"git_status"}]}])).is_err());
        assert!(declarations(&json!([{"repository":[{"kind":"git_status"}]}])).is_err());
        assert!(
            declarations(&json!([{"manifestId":"api","dependencies":[{"kind":"grype_scan"}]}]))
                .is_err()
        );
        assert!(declarations(&json!([{"manifestId":"api","checks":[{"kind":"ruff","id":"unit"}],"tests":[{"kind":"pytest","id":"unit"}]}])).is_ok());
        assert!(declarations(&json!([{"manifestId":"api","dependencies":[{"kind":"syft_inventory"},{"kind":"grype_scan"}]}])).is_ok());
        assert!(declarations(&json!([{"manifestId":"api","dependencies":[{"kind":"grype_scan","sbom":{"collector_id":"inventory","manifest_id":"shared"}}]},{"manifestId":"shared","dependencies":[{"kind":"syft_inventory","id":"inventory"}]}])).is_ok());
    }
}
