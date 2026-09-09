//! Tenant-scoped, append-only workflow history independent of generated artifacts.

use super::{DatabasePool, schema};
use crate::action_history::{ActionStatusEvent, GithubActionRun, GithubActionsLog};
use crate::manifest::ManifestRequestContext;
use chrono::{SecondsFormat, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

pub(super) const CREATE_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS manifest_action_events (
    clerk_org_id TEXT NOT NULL,
    manifest_id TEXT NOT NULL,
    run_key TEXT NOT NULL,
    event_id TEXT NOT NULL,
    status TEXT NOT NULL,
    conclusion TEXT NOT NULL,
    source_updated_at TEXT NOT NULL,
    phase INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    run_id INTEGER NOT NULL,
    run_attempt INTEGER NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY (clerk_org_id, manifest_id, run_key, event_id),
    UNIQUE (clerk_org_id, manifest_id, run_key, status, conclusion, source_updated_at)
)";

const INSERT_EVENT: &str = r"
INSERT INTO manifest_action_events (
    clerk_org_id, manifest_id, run_key, event_id, status, conclusion,
    source_updated_at, phase, created_at, run_id, run_attempt, content
) SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
WHERE NOT (? = 'api' AND EXISTS (
    SELECT 1 FROM (
        SELECT status, conclusion, source_updated_at FROM manifest_action_events
        WHERE clerk_org_id = ? AND manifest_id = ? AND run_key = ?
        ORDER BY source_updated_at DESC, phase DESC, event_id DESC LIMIT 1
    ) WHERE status = ? AND conclusion = ? AND source_updated_at <= ?
)) ON CONFLICT DO NOTHING";

const READ_EVENTS: &str = r"
WITH recent_runs AS (
    SELECT run_key FROM manifest_action_events
    WHERE clerk_org_id = ? AND manifest_id = ?
    GROUP BY run_key
    ORDER BY MAX(created_at) DESC, MAX(run_id) DESC, MAX(run_attempt) DESC, run_key
    LIMIT ? OFFSET ?
)
SELECT content FROM manifest_action_events
WHERE clerk_org_id = ? AND manifest_id = ? AND run_key IN (SELECT run_key FROM recent_runs)
ORDER BY source_updated_at ASC, phase ASC, event_id ASC";

/// Validate an explicit identity that is stable across source edits.
fn validate_manifest_id(manifest_id: &str) -> Result<(), String> {
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

/// Append an observation atomically; duplicates return false.
///
/// # Errors
/// Returns an error for unauthorized writes, invalid data, or unavailable storage.
pub async fn record_action_run(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    manifest_id: &str,
    mut run: GithubActionRun,
    event_id: Option<String>,
    source: &str,
) -> Result<bool, String> {
    if context.clerk_org_id.is_empty() || !context.can_write_generated_manifests() {
        return Err("active organization role cannot record action history".into());
    }
    validate_manifest_id(manifest_id)?;
    run.validate()?;
    if !matches!(source, "api" | "webhook" | "manual") {
        return Err("source must be api, webhook, or manual".into());
    }
    run.host = run.host.to_lowercase();
    let key = run.identity();
    let updated = run.updated_at.to_rfc3339_opts(SecondsFormat::Nanos, true);
    let conclusion = run.conclusion.clone().unwrap_or_default();
    let fingerprint = serde_json::to_vec(&(&key, &run.status, &conclusion, &updated))
        .map_err(|error| error.to_string())?;
    let event_id = event_id.unwrap_or_else(|| {
        Sha256::digest(fingerprint)
            .iter()
            .fold(String::with_capacity(64), |mut output, byte| {
                let _ = write!(output, "{byte:02x}");
                output
            })
    });
    if event_id.trim().is_empty() || event_id.len() > 256 {
        return Err("eventId must contain 1 to 256 characters".into());
    }
    let event = ActionStatusEvent {
        event_id: event_id.clone(),
        status: run.status.clone(),
        conclusion: run.conclusion.clone(),
        source_updated_at: run.updated_at,
        recorded_at: Utc::now(),
        source: source.into(),
    };
    let phase = event.order_key().1.to_string();
    // The server assigns receipt time and accepts exactly one observation per request.
    run.events = vec![event];
    let snapshot = serde_json::to_string(&run).map_err(|error| error.to_string())?;
    let parameters = vec![
        context.clerk_org_id.clone(),
        manifest_id.into(),
        key.clone(),
        event_id,
        run.status.clone(),
        conclusion.clone(),
        updated.clone(),
        phase,
        run.created_at.to_rfc3339_opts(SecondsFormat::Nanos, true),
        run.run_id.to_string(),
        run.run_attempt.to_string(),
        snapshot,
        source.into(),
        context.clerk_org_id.clone(),
        manifest_id.into(),
        key,
        run.status,
        conclusion,
        updated,
    ];
    schema::ensure_table(pool).await?;
    let affected = match pool {
        DatabasePool::Sqlite(pool) => {
            let mut query = sqlx::query(INSERT_EVENT);
            for value in &parameters {
                query = query.bind(value);
            }
            query
                .execute(pool)
                .await
                .map_err(|error| error.to_string())?
                .rows_affected()
        }
        DatabasePool::Turso(database) => database
            .connect()
            .map_err(|error| error.to_string())?
            .execute(
                INSERT_EVENT,
                parameters
                    .into_iter()
                    .map(libsql::Value::Text)
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(|error| error.to_string())?,
    };
    Ok(affected > 0)
}

/// Read a page of complete workflow attempts, newest first.
///
/// # Errors
/// Returns an error for invalid pagination, invalid identity, or unavailable storage.
pub async fn read_action_history(
    pool: &DatabasePool,
    clerk_org_id: &str,
    manifest_id: &str,
    limit: u32,
    offset: u32,
) -> Result<GithubActionsLog, String> {
    validate_manifest_id(manifest_id)?;
    if !(1..=100).contains(&limit) || clerk_org_id.is_empty() {
        return Err("history requires an organization and a limit between 1 and 100".into());
    }
    schema::ensure_table(pool).await?;
    let parameters = [
        clerk_org_id.to_string(),
        manifest_id.into(),
        limit.to_string(),
        offset.to_string(),
        clerk_org_id.into(),
        manifest_id.into(),
    ];
    let contents: Vec<String> = match pool {
        DatabasePool::Sqlite(pool) => {
            let mut query = sqlx::query_scalar(READ_EVENTS);
            for value in &parameters {
                query = query.bind(value);
            }
            query
                .fetch_all(pool)
                .await
                .map_err(|error| error.to_string())?
        }
        DatabasePool::Turso(database) => {
            let connection = database.connect().map_err(|error| error.to_string())?;
            let mut rows = connection
                .query(
                    READ_EVENTS,
                    parameters
                        .into_iter()
                        .map(libsql::Value::Text)
                        .collect::<Vec<_>>(),
                )
                .await
                .map_err(|error| error.to_string())?;
            let mut contents = Vec::new();
            while let Some(row) = rows.next().await.map_err(|error| error.to_string())? {
                contents.push(row.get(0).map_err(|error| error.to_string())?);
            }
            contents
        }
    };
    let mut log = GithubActionsLog::default();
    for content in contents {
        log.merge(serde_json::from_str(&content).map_err(|error| error.to_string())?);
    }
    Ok(log)
}

/// Attach recent durable history when reading generated Manifest values.
pub(super) async fn attach_history(
    pool: &DatabasePool,
    clerk_org_id: &str,
    mut value: Value,
) -> Result<Value, String> {
    if let Some(manifests) = value.as_array_mut() {
        for manifest in manifests {
            let Some(id) = manifest.get("manifestId").and_then(Value::as_str) else {
                continue;
            };
            let durable = read_action_history(pool, clerk_org_id, id, 100, 0).await?;
            if durable.runs.is_empty() {
                continue;
            }
            if manifest.get("cicd").is_none_or(Value::is_null) {
                manifest["cicd"] = serde_json::json!({});
            }
            let mut log: GithubActionsLog = manifest["cicd"]
                .get("githubActions")
                .filter(|value| !value.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|error| error.to_string())?
                .unwrap_or_default();
            for run in durable.runs {
                log.merge(run);
            }
            manifest["cicd"]["buildStatus"] = serde_json::json!(log.build_status());
            if let Some(latest) = log
                .runs
                .iter()
                .max_by_key(|run| (run.created_at, run.run_id, run.run_attempt))
            {
                manifest["cicd"]["lastBuild"] = serde_json::json!(latest.updated_at);
            }
            manifest["cicd"]["githubActions"] =
                serde_json::to_value(log).map_err(|error| error.to_string())?;
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests;
