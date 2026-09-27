//! Monotonic provider-run state, separate from immutable local delivery history.
use super::{
    DatabasePool, EvidenceObservation, content, deserialize, query, time, transaction::Transaction,
};
use crate::{
    collectors::fingerprint,
    evidence::{EvidenceResult, WorkflowRun},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(in crate::persistence) const CREATE: &str = "CREATE TABLE workflow_run_projection (identity TEXT PRIMARY KEY, org TEXT NOT NULL, manifest_id TEXT NOT NULL, collector_id TEXT NOT NULL, workspace_id TEXT NOT NULL, source_key TEXT NOT NULL, source_updated_at TEXT NOT NULL, lifecycle INTEGER NOT NULL, fingerprint TEXT NOT NULL, content TEXT NOT NULL)";
pub(in crate::persistence) const INDEX: &str = "CREATE INDEX workflow_projection_source ON workflow_run_projection(source_key,source_updated_at DESC,identity)";

pub(in crate::persistence) const OWNERSHIP_INDEX: &str = "CREATE INDEX workflow_projection_owner ON workflow_run_projection(org,manifest_id,collector_id,workspace_id,source_key)";

#[derive(Serialize, Deserialize)]
struct Projection {
    identity: String,
    run: WorkflowRun,
}
fn source_key(
    org: &str,
    observation: &EvidenceObservation,
    repository: &str,
) -> Result<String, String> {
    fingerprint(&(
        org,
        &observation.manifest_id,
        observation.section,
        &observation.collector_id,
        &observation.collector_revision,
        &observation.workspace_id,
        &observation.environment,
        &observation.scope,
        repository,
    ))
}
pub(super) fn key_for_observation(
    org: &str,
    observation: &EvidenceObservation,
) -> Result<String, String> {
    match &observation.result {
        EvidenceResult::Workflows(result) => source_key(org, observation, &result.repository),
        _ => Ok(String::new()),
    }
}
fn identity(source: &str, run: &WorkflowRun) -> Result<String, String> {
    fingerprint(&(source, &run.run_id, run.attempt))
}
fn lifecycle(run: &WorkflowRun) -> u8 {
    match run.status.as_str() {
        "completed" => 2,
        "in_progress" => 1,
        _ => 0,
    }
}
pub(super) async fn record(
    tx: &mut Transaction<'_>,
    org: &str,
    observation: &EvidenceObservation,
) -> Result<(), String> {
    let EvidenceResult::Workflows(result) = &observation.result else {
        return Ok(());
    };
    let source = source_key(org, observation, &result.repository)?;
    for run in &result.items {
        let identity = identity(&source, run)?;
        let projection = Projection {
            identity: identity.clone(),
            run: run.clone(),
        };
        // Lifecycle cannot go backwards within one provider attempt. Provider time
        // wins first; equal-time terminal states win, then a stable content hash.
        tx.execute("INSERT INTO workflow_run_projection (identity,org,manifest_id,collector_id,workspace_id,source_key,source_updated_at,lifecycle,fingerprint,content) VALUES (?,?,?,?,?,?,?,?,?,?) ON CONFLICT(identity) DO UPDATE SET source_updated_at=excluded.source_updated_at,lifecycle=excluded.lifecycle,fingerprint=excluded.fingerprint,content=excluded.content WHERE (excluded.source_updated_at>workflow_run_projection.source_updated_at AND excluded.lifecycle>=workflow_run_projection.lifecycle) OR (excluded.source_updated_at=workflow_run_projection.source_updated_at AND (excluded.lifecycle>workflow_run_projection.lifecycle OR (excluded.lifecycle=workflow_run_projection.lifecycle AND excluded.fingerprint>workflow_run_projection.fingerprint)))",vec![identity,org.into(),observation.manifest_id.clone(),observation.collector_id.clone(),observation.workspace_id.clone(),source.clone(),time(run.updated_at),lifecycle(run).to_string(),fingerprint(run)?,content(&projection)?]).await?;
    }
    // Keep a bounded set per exact source, including distinct rerun attempts.
    tx.execute("DELETE FROM workflow_run_projection WHERE identity IN (SELECT identity FROM workflow_run_projection WHERE source_key=? ORDER BY source_updated_at DESC,identity DESC LIMIT -1 OFFSET 1000)",vec![source]).await?;
    Ok(())
}
pub(super) async fn project(
    pool: &DatabasePool,
    org: &str,
    observations: &mut [EvidenceObservation],
) -> Result<(), String> {
    let mut requested = Vec::new();
    for observation in observations.iter() {
        if let EvidenceResult::Workflows(result) = &observation.result {
            let source = source_key(org, observation, &result.repository)?;
            for run in &result.items {
                requested.push(identity(&source, run)?);
            }
        }
    }
    requested.sort();
    requested.dedup();
    let mut canonical = BTreeMap::new();
    // Only displayed run identities are read; no observation history is scanned.
    for ids in requested.chunks(100) {
        let slots = vec!["?"; ids.len()].join(",");
        let rows: Vec<Projection> = deserialize(
            query(
                pool,
                &format!("SELECT content FROM workflow_run_projection WHERE identity IN ({slots})"),
                ids.to_vec(),
            )
            .await?,
        )?;
        canonical.extend(rows.into_iter().map(|row| (row.identity, row.run)));
    }
    for observation in observations {
        let source = match &observation.result {
            EvidenceResult::Workflows(result) => source_key(org, observation, &result.repository)?,
            _ => continue,
        };
        if let EvidenceResult::Workflows(result) = &mut observation.result {
            for run in &mut result.items {
                if let Some(current) = canonical.get(&identity(&source, run)?) {
                    *run = current.clone();
                }
            }
        }
    }
    Ok(())
}

/// Drop derived states once retention removes their last immutable source snapshot.
pub(super) async fn prune(
    tx: &mut Transaction<'_>,
    org: &str,
    observation: &EvidenceObservation,
) -> Result<(), String> {
    tx.execute("DELETE FROM workflow_run_projection AS projection WHERE org=? AND manifest_id=? AND collector_id=? AND workspace_id=? AND NOT EXISTS (SELECT 1 FROM evidence_observations WHERE workflow_source_key=projection.source_key)",vec![org.into(),observation.manifest_id.clone(),observation.collector_id.clone(),observation.workspace_id.clone()]).await?;
    Ok(())
}
