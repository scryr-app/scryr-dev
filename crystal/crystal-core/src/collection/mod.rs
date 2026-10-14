//! Declarative local collection plans and their persisted observations.
mod runtime;
mod schema;

use async_graphql::{Json, SimpleObject};
pub use runtime::CollectionRuntime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// Versioned execution graph exported by the Python SDK.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionPlan {
    /// Wire format version.
    pub version: u32,
    /// Reusable executable operations.
    pub tasks: Vec<Task>,
    /// Scheduled task bindings.
    pub collectors: Vec<Collector>,
    /// Read-only views over collector results.
    pub cards: Vec<Card>,
}
impl Default for CollectionPlan {
    fn default() -> Self {
        Self {
            version: 1,
            tasks: vec![],
            collectors: vec![],
            cards: vec![],
        }
    }
}
/// A command and its portable output contract.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    /// Stable task identifier.
    pub id: String,
    /// Direct subprocess invocation.
    pub command: Command,
    /// Supported JSON Schema output contract.
    pub output_schema: Value,
    /// Execution deadline, including output reads.
    pub timeout_seconds: u32,
    /// Maximum bytes allowed on each output stream.
    pub max_output_bytes: u32,
    /// Exit codes that permit output parsing.
    pub accepted_exit_codes: Vec<i32>,
}
/// An argument-array command, without implicit shell interpretation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    /// Executable followed by its arguments.
    pub argv: Vec<String>,
    /// Working directory relative to the manifest directory.
    pub cwd: String,
}
/// A local scheduled instance of a task.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collector {
    /// Stable collector identifier.
    pub id: String,
    /// Referenced task identifier.
    pub task_id: String,
    /// User-facing environment label.
    pub environment: String,
    /// Period between executions.
    pub interval_seconds: u32,
    /// Age at which successful data becomes stale.
    pub stale_after_seconds: i32,
    /// Maximum persisted observation age.
    pub retain_for_seconds: u32,
}
/// Declarative card binding.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    /// Stable card identifier.
    pub id: String,
    /// Presentation primitive: status, table, or history.
    pub kind: String,
    /// Display title.
    pub title: String,
    /// Source collector identifier.
    pub collector_id: String,
    /// Explicit architecture manifest identifier.
    pub target: String,
}
/// A persisted execution outcome; failures never expose raw process output.
#[derive(Clone, Debug, Serialize, SimpleObject)]
pub struct Observation {
    /// Unique run identifier.
    pub id: String,
    /// Source collector identifier.
    pub collector_id: String,
    /// Execution start in UTC.
    pub started_at: String,
    /// Execution end in UTC.
    pub finished_at: String,
    /// success, failed, timeout, or `invalid_output`.
    pub status: String,
    /// Process exit code, when available.
    pub exit_code: Option<i32>,
    /// Sanitized collection failure description.
    pub error: Option<String>,
    /// Validated JSON report on successful collection.
    pub data: Option<Json<Value>>,
    /// Digest of the active collection plan.
    pub plan_revision: String,
}
/// A read-only card with shared collector observations.
#[derive(Clone, Debug, Serialize, SimpleObject)]
pub struct CollectorCard {
    /// Stable card identifier.
    pub id: String,
    /// Presentation primitive.
    pub kind: String,
    /// Display title.
    pub title: String,
    /// Architecture manifest identifier.
    pub target: String,
    /// Source collector identifier.
    pub collector_id: String,
    /// Environment label.
    pub environment: String,
    /// Successful data freshness threshold.
    pub stale_after_seconds: i32,
    /// Latest execution outcome.
    pub latest: Option<Observation>,
    /// Latest valid report from this plan revision.
    pub last_success: Option<Observation>,
    /// Up to twenty recent executions, newest first.
    pub history: Vec<Observation>,
}
impl CollectionPlan {
    /// Validate all references and the portable schema subset before execution.
    ///
    /// # Errors
    /// Returns a descriptive error for unsupported or inconsistent plans.
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported collection plan version".into());
        }
        let mut tasks = HashSet::new();
        for task in &self.tasks {
            if task.id.trim().is_empty() || !tasks.insert(&task.id) {
                return Err("task IDs must be nonempty and unique".into());
            }
            if task.command.argv.is_empty()
                || task.command.argv[0].is_empty()
                || task.command.argv.iter().any(|arg| arg.contains('\0'))
            {
                return Err(format!("task {} has invalid argv", task.id));
            }
            if task.timeout_seconds == 0
                || task.timeout_seconds > 86400
                || task.max_output_bytes == 0
                || task.max_output_bytes > 16 * 1024 * 1024
                || task.accepted_exit_codes.is_empty()
            {
                return Err(format!("task {} has invalid execution limits", task.id));
            }
            schema::validate_schema(&task.output_schema)?;
        }
        let mut collectors = HashSet::new();
        for collector in &self.collectors {
            if collector.id.trim().is_empty()
                || !collectors.insert(&collector.id)
                || !tasks.contains(&collector.task_id)
            {
                return Err("collector IDs must be unique and reference an existing task".into());
            }
            if collector.interval_seconds == 0
                || collector.stale_after_seconds <= 0
                || collector.retain_for_seconds == 0
            {
                return Err("collector intervals and retention must be positive".into());
            }
        }
        let mut cards = HashSet::new();
        for card in &self.cards {
            if card.id.trim().is_empty()
                || !cards.insert(&card.id)
                || !collectors.contains(&card.collector_id)
                || card.target.trim().is_empty()
                || !matches!(card.kind.as_str(), "status" | "table" | "history")
            {
                return Err("invalid card ID, target, kind, or collector reference".into());
            }
        }
        Ok(())
    }
}
