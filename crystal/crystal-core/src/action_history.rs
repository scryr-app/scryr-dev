//! Workflow attempts and their append-only observations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An observed workflow state, separate from its receipt time.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionStatusEvent {
    /// Retry-stable delivery or observation identifier.
    pub event_id: String,
    /// GitHub workflow status.
    pub status: String,
    /// GitHub workflow conclusion, when available.
    pub conclusion: Option<String>,
    /// Timestamp supplied by GitHub.
    pub source_updated_at: DateTime<Utc>,
    /// Timestamp at which Scryr recorded this observation.
    pub recorded_at: DateTime<Utc>,
    /// Collection mechanism: api, webhook, or manual.
    pub source: String,
}

impl ActionStatusEvent {
    /// Deterministic order for same-timestamp deliveries.
    #[must_use]
    pub fn order_key(&self) -> (DateTime<Utc>, u8, &str) {
        let rank = match self.status.as_str() {
            "completed" => 2,
            "in_progress" => 1,
            _ => 0,
        };
        (self.source_updated_at, rank, &self.event_id)
    }
}

/// One workflow run attempt and its observed transitions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubActionRun {
    /// GitHub web host, including enterprise hosts.
    pub host: String,
    /// Stable repository identity.
    pub repository_id: u64,
    /// Repository owner/name.
    pub repository: String,
    /// Stable workflow identity.
    pub workflow_id: u64,
    /// Workflow display name.
    pub workflow_name: String,
    /// Stable workflow run identity.
    pub run_id: u64,
    /// Attempt number, starting at one.
    pub run_attempt: u64,
    /// Branch on which the run was triggered.
    pub head_branch: Option<String>,
    /// Exact commit that ran.
    pub head_sha: String,
    /// Human-readable workflow run URL.
    pub html_url: String,
    /// Current provider status.
    pub status: String,
    /// Current provider conclusion.
    pub conclusion: Option<String>,
    /// Provider creation time.
    pub created_at: DateTime<Utc>,
    /// Provider update time.
    pub updated_at: DateTime<Utc>,
    /// Provider execution start time.
    pub run_started_at: Option<DateTime<Utc>>,
    /// API or archived log reference; console text is not embedded.
    pub logs_url: Option<String>,
    /// Observations ordered by provider time, phase, and event ID.
    #[serde(default)]
    pub events: Vec<ActionStatusEvent>,
}

impl GithubActionRun {
    /// Validate a provider observation before accepting it into storage.
    ///
    /// # Errors
    /// Returns an error for malformed identities, states, or timestamps.
    pub fn validate(&self) -> Result<(), String> {
        if self.host.trim().is_empty()
            || self.repository_id == 0
            || self.workflow_id == 0
            || self.run_id == 0
            || self.run_attempt == 0
            || self.head_sha.trim().is_empty()
            || self.repository.split('/').count() != 2
            || self
                .repository
                .split('/')
                .any(|part| part.is_empty() || part.chars().any(char::is_whitespace))
            || !(self.html_url.starts_with("https://") || self.html_url.starts_with("http://"))
        {
            return Err("invalid GitHub run identity".into());
        }
        if !matches!(
            self.status.as_str(),
            "queued" | "in_progress" | "completed" | "waiting" | "pending" | "requested"
        ) {
            return Err("invalid GitHub run status".into());
        }
        if let Some(conclusion) = self.conclusion.as_deref()
            && (self.status != "completed"
                || !matches!(
                    conclusion,
                    "success"
                        | "failure"
                        | "cancelled"
                        | "neutral"
                        | "skipped"
                        | "stale"
                        | "timed_out"
                        | "action_required"
                        | "startup_failure"
                ))
        {
            return Err("invalid GitHub run conclusion".into());
        }
        if self.updated_at < self.created_at {
            return Err("updatedAt must not precede createdAt".into());
        }
        Ok(())
    }

    /// Order a snapshot even when an imported run has no explicit events.
    #[must_use]
    pub fn order_key(&self) -> (DateTime<Utc>, u8, &str) {
        let rank = match self.status.as_str() {
            "completed" => 2,
            "in_progress" => 1,
            _ => 0,
        };
        let event_id = self
            .events
            .iter()
            .max_by_key(|event| event.order_key())
            .map_or("", |event| event.event_id.as_str());
        (self.updated_at, rank, event_id)
    }

    /// Stable storage key that keeps attempts and hosts separate.
    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.host.to_lowercase(),
            self.repository_id,
            self.run_id,
            self.run_attempt
        )
    }
}

/// Workflow attempts attached to a Manifest.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GithubActionsLog {
    /// Runs ordered newest first.
    pub runs: Vec<GithubActionRun>,
}

impl GithubActionsLog {
    /// Summarize the newest reported build across workflows. Reporters select the branch.
    #[must_use]
    pub fn build_status(&self) -> Option<&'static str> {
        let run = self
            .runs
            .iter()
            .max_by_key(|run| (run.created_at, run.run_id, run.run_attempt))?;
        if run.status != "completed" {
            return Some("pending");
        }
        match run.conclusion.as_deref() {
            Some("success") => Some("passing"),
            Some("failure" | "timed_out" | "action_required" | "startup_failure") => {
                Some("failing")
            }
            _ => None,
        }
    }

    /// Merge a stored snapshot without allowing late delivery to regress current state.
    pub fn merge(&mut self, mut run: GithubActionRun) {
        if let Some(current) = self
            .runs
            .iter_mut()
            .find(|item| item.identity() == run.identity())
        {
            let incoming_wins = run.order_key() > current.order_key();
            let mut events = std::mem::take(&mut current.events);
            for event in &run.events {
                if !events.iter().any(|item| item.event_id == event.event_id) {
                    events.push(event.clone());
                }
            }
            events.sort_by(|left, right| left.order_key().cmp(&right.order_key()));
            if incoming_wins {
                *current = run;
            }
            current.events = events;
        } else {
            run.events
                .sort_by(|left, right| left.order_key().cmp(&right.order_key()));
            self.runs.push(run);
        }
        self.runs
            .sort_by_key(|run| std::cmp::Reverse((run.created_at, run.run_id, run.run_attempt)));
    }
}
