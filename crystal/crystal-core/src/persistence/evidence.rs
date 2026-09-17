//! Tenant-scoped immutable observations and independently updated collector status.
use super::{DatabasePool, schema};
use crate::{
    collectors::CollectorDeclaration,
    evidence::{
        CollectorEvidence, CollectorState, CollectorStatus, EvidenceObservation, EvidenceSection,
    },
    manifest::ManifestRequestContext,
};
use chrono::Utc;

mod transaction;
pub(super) mod workflows;

pub(super) const CREATE_OBSERVATIONS: &str = "CREATE TABLE IF NOT EXISTS evidence_observations (org TEXT NOT NULL, observation_id TEXT NOT NULL, manifest_id TEXT NOT NULL, section TEXT NOT NULL, collector_id TEXT NOT NULL, workspace_id TEXT NOT NULL, collector_revision TEXT NOT NULL, environment TEXT NOT NULL, scope TEXT NOT NULL, workflow_source_key TEXT NOT NULL DEFAULT '', observed_at TEXT NOT NULL, attempt INTEGER NOT NULL, recorded_at TEXT NOT NULL, fingerprint TEXT NOT NULL, content TEXT NOT NULL, PRIMARY KEY(org,observation_id))";
pub(super) const CREATE_STATUSES: &str = "CREATE TABLE IF NOT EXISTS collector_status (org TEXT NOT NULL, manifest_id TEXT NOT NULL, section TEXT NOT NULL, collector_id TEXT NOT NULL, workspace_id TEXT NOT NULL, collector_revision TEXT NOT NULL, updated_at TEXT NOT NULL, content TEXT NOT NULL, PRIMARY KEY(org,manifest_id,section,collector_id,workspace_id,collector_revision))";
pub(super) const CREATE_INDEX: &str = "CREATE INDEX IF NOT EXISTS evidence_lookup ON evidence_observations (org,manifest_id,section,collector_id,workspace_id,collector_revision,environment,scope,observed_at DESC,attempt DESC,observation_id DESC)";

pub(super) const WORKFLOW_SOURCES_INDEX: &str =
    "CREATE INDEX evidence_workflow_source ON evidence_observations(workflow_source_key)";

fn authorize(context: &ManifestRequestContext) -> Result<(), String> {
    if context.clerk_org_id.is_empty() || !context.can_write_generated_manifests() {
        Err("active organization role cannot record evidence".into())
    } else {
        Ok(())
    }
}
fn time(value: chrono::DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}
fn content<T: serde::Serialize>(value: &T) -> Result<String, String> {
    let value = serde_json::to_string(value).map_err(|e| e.to_string())?;
    if value.len() > 2_000_000 {
        return Err("evidence exceeds 2 MB".into());
    }
    Ok(value)
}
/// Record one immutable observation. Identical delivery retries return false.
/// # Errors
/// Rejects unauthorized, invalid, conflicting, or oversized observations and storage failures.
pub async fn record_evidence(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    mut observation: EvidenceObservation,
) -> Result<bool, String> {
    authorize(context)?;
    observation.validate()?;
    observation.recorded_at = None;
    let fingerprint = crate::collectors::fingerprint(&observation)?;
    observation.recorded_at = Some(Utc::now());
    let serialized = content(&observation)?;
    let retention = vec![
        context.clerk_org_id.clone(),
        observation.manifest_id.clone(),
        observation.section.as_str().into(),
        observation.collector_id.clone(),
        observation.workspace_id.clone(),
    ];
    let retention_hours = if observation.section == EvidenceSection::Metrics {
        24
    } else {
        24 * 30
    };
    schema::ensure_table(pool).await?;
    let mut tx = transaction::Transaction::begin(pool).await?;
    let count=tx.execute("INSERT INTO evidence_observations (org,observation_id,manifest_id,section,collector_id,workspace_id,collector_revision,environment,scope,workflow_source_key,observed_at,attempt,recorded_at,fingerprint,content) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT DO NOTHING",vec![context.clerk_org_id.clone(),observation.observation_id.clone(),observation.manifest_id.clone(),observation.section.as_str().into(),observation.collector_id.clone(),observation.workspace_id.clone(),observation.collector_revision.clone(),observation.environment.clone(),observation.scope.clone(),workflows::key_for_observation(&context.clerk_org_id,&observation)?,time(observation.observed_at),observation.attempt.to_string(),time(observation.recorded_at.ok_or("missing receipt time")?),fingerprint.clone(),serialized]).await?;
    if count == 0 {
        let existing = tx
            .query(
                "SELECT fingerprint FROM evidence_observations WHERE org=? AND observation_id=?",
                vec![
                    context.clerk_org_id.clone(),
                    observation.observation_id.clone(),
                ],
            )
            .await?;
        if existing.first() != Some(&fingerprint) {
            return Err(
                "observationId is immutable and already contains different evidence".into(),
            );
        }
    }
    workflows::record(&mut tx, &context.clerk_org_id, &observation).await?;
    let cutoff = time(Utc::now() - chrono::Duration::hours(retention_hours));
    let mut retention = retention;
    retention.push(cutoff);
    // Retain the newest result for every source scope even after its age limit.
    tx.execute("DELETE FROM evidence_observations WHERE (org,observation_id) IN (SELECT org,observation_id FROM (SELECT org,observation_id,observed_at,ROW_NUMBER() OVER (PARTITION BY org,manifest_id,section,collector_id,workspace_id,environment,scope ORDER BY observed_at DESC,attempt DESC,observation_id DESC) AS rank FROM evidence_observations WHERE org=? AND manifest_id=? AND section=? AND collector_id=? AND workspace_id=?) WHERE rank>1 AND (observed_at<? OR rank>1000))",retention).await?;
    workflows::prune(&mut tx, &context.clerk_org_id, &observation).await?;
    tx.commit().await?;
    Ok(count > 0)
}
/// Replace status only when it is newer than the current status for that revision.
/// # Errors
/// Rejects unauthorized or invalid status and storage failures.
pub async fn record_collector_status(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    status: CollectorStatus,
) -> Result<bool, String> {
    authorize(context)?;
    status.validate()?;
    let serialized = content(&status)?;
    schema::ensure_table(pool).await?;
    let count=execute(pool,"INSERT INTO collector_status (org,manifest_id,section,collector_id,workspace_id,collector_revision,updated_at,content) VALUES (?,?,?,?,?,?,?,?) ON CONFLICT(org,manifest_id,section,collector_id,workspace_id,collector_revision) DO UPDATE SET updated_at=excluded.updated_at,content=excluded.content WHERE excluded.updated_at>collector_status.updated_at",vec![context.clerk_org_id.clone(),status.manifest_id,status.section.as_str().into(),status.collector_id,status.workspace_id,status.collector_revision,time(status.updated_at),serialized]).await?;
    Ok(count > 0)
}
/// Read a collector's history, newest source observation first.
/// # Errors
/// Rejects malformed identities/pagination and storage failures.
#[allow(clippy::too_many_arguments)]
pub async fn evidence_history(
    pool: &DatabasePool,
    org: &str,
    manifest_id: &str,
    section: EvidenceSection,
    collector_id: &str,
    workspace_id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<EvidenceObservation>, String> {
    for id in [org, manifest_id, collector_id, workspace_id] {
        crate::evidence::validate_identity(id)?;
    }
    if !(1..=100).contains(&limit) {
        return Err("history limit must be between 1 and 100".into());
    }
    schema::ensure_table(pool).await?;
    let rows=query(pool,"SELECT content FROM evidence_observations WHERE org=? AND manifest_id=? AND section=? AND collector_id=? AND workspace_id=? ORDER BY observed_at DESC,attempt DESC,observation_id DESC LIMIT ? OFFSET ?",vec![org.into(),manifest_id.into(),section.as_str().into(),collector_id.into(),workspace_id.into(),limit.to_string(),offset.to_string()]).await?;
    deserialize(rows)
}
/// Read one immutable observation without resolving current declaration state.
/// # Errors
/// Rejects malformed identities and storage failures.
pub async fn evidence_observation(
    pool: &DatabasePool,
    org: &str,
    observation_id: &str,
    workspace_id: Option<&str>,
) -> Result<Option<EvidenceObservation>, String> {
    for id in [org, observation_id] {
        crate::evidence::validate_identity(id)?;
    }
    if let Some(id) = workspace_id {
        crate::evidence::validate_identity(id)?;
    }
    schema::ensure_table(pool).await?;
    let mut params = vec![org.into(), observation_id.into()];
    let sql = workspace_id.map_or(
        "SELECT content FROM evidence_observations WHERE org=? AND observation_id=?",
        |workspace| {
            params.push(workspace.into());
            "SELECT content FROM evidence_observations WHERE org=? AND observation_id=? AND workspace_id=?"
        },
    );
    let mut rows = deserialize(query(pool, sql, params).await?)?;
    Ok(rows.pop())
}

/// Project declarations onto current evidence without altering stored manifest configuration.
/// Reads are batched across manifests; a single workspace is selected for the entire request.
/// # Errors
/// Returns validation and storage errors.
#[allow(clippy::too_many_lines)] // One batched read and deterministic projection in declaration order.
pub async fn project_evidence(
    pool: &DatabasePool,
    org: &str,
    declarations: &[CollectorDeclaration],
    workspace_id: Option<&str>,
) -> Result<Vec<CollectorEvidence>, String> {
    crate::evidence::validate_identity(org)?;
    if let Some(workspace) = workspace_id {
        crate::evidence::validate_identity(workspace)?;
    }
    if declarations.is_empty() {
        return Ok(vec![]);
    }
    schema::ensure_table(pool).await?;
    let mut ids: Vec<_> = declarations.iter().map(|d| d.manifest_id.clone()).collect();
    ids.extend(
        declarations
            .iter()
            .filter_map(|d| d.config.sbom().and_then(|r| r.manifest_id.clone())),
    );
    ids.sort();
    ids.dedup();
    let slots = vec!["?"; ids.len()].join(",");
    let workspace_clause = if workspace_id.is_some() {
        " AND workspace_id=?"
    } else {
        ""
    };
    let mut params = vec![org.into()];
    params.extend(ids);
    if let Some(w) = workspace_id {
        params.push(w.into());
    }
    let latest_sql = format!(
        "SELECT content FROM (SELECT content,ROW_NUMBER() OVER (PARTITION BY manifest_id,section,collector_id,workspace_id,collector_revision,environment,scope ORDER BY observed_at DESC,attempt DESC,observation_id DESC) AS rank FROM evidence_observations WHERE org=? AND manifest_id IN ({slots}){workspace_clause}) WHERE rank=1"
    );
    let mut observations: Vec<EvidenceObservation> =
        deserialize(query(pool, &latest_sql, params.clone()).await?)?;
    workflows::project(pool, org, &mut observations).await?;
    let statuses:Vec<CollectorStatus>=deserialize(query(pool,&format!("SELECT content FROM collector_status WHERE org=? AND manifest_id IN ({slots}){workspace_clause}"),params).await?)?;
    let selected = workspace_id
        .map(str::to_owned)
        .or_else(|| {
            observations
                .iter()
                .map(|v| (v.recorded_at.unwrap_or(v.observed_at), &v.workspace_id))
                .chain(statuses.iter().map(|v| (v.updated_at, &v.workspace_id)))
                .max()
                .map(|(_, s)| s.clone())
        })
        .unwrap_or_else(|| "local".into());
    let mut result = Vec::with_capacity(declarations.len());
    for d in declarations {
        let matches = |m: &str, s: EvidenceSection, c: &str, w: &str| {
            m == d.manifest_id && s == d.section && c == d.config.id() && w == selected
        };
        let latest = observations
            .iter()
            .filter(|v| matches(&v.manifest_id, v.section, &v.collector_id, &v.workspace_id))
            .max_by_key(|v| {
                (
                    v.collector_revision == d.revision,
                    v.observed_at,
                    v.attempt,
                    &v.observation_id,
                )
            })
            .cloned();
        let status = statuses
            .iter()
            .filter(|v| {
                matches(&v.manifest_id, v.section, &v.collector_id, &v.workspace_id)
                    && v.collector_revision == d.revision
            })
            .max_by_key(|v| v.updated_at);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let freshness = d.config.common().freshness.ceil() as u64;
        let outdated = latest.as_ref().is_some_and(|v| {
            v.collector_revision != d.revision
                || status
                    .and_then(|s| s.input_fingerprint.as_ref())
                    .is_some_and(|f| *f != v.input_fingerprint)
                || upstream_outdated(d, v, &observations, declarations, &selected)
        });
        let stale = status.is_some_and(|s| {
            matches!(
                s.state,
                CollectorState::Error
                    | CollectorState::MissingTool
                    | CollectorState::IncompatibleTool
                    | CollectorState::NeedsLogin
            )
        }) || latest.as_ref().is_some_and(|v| {
            Utc::now()
                .signed_duration_since(v.source_updated_at.unwrap_or(v.observed_at))
                .num_seconds()
                > i64::try_from(freshness).unwrap_or(i64::MAX)
        });
        result.push(CollectorEvidence {
            manifest_id: d.manifest_id.clone(),
            section: d.section,
            collector_id: d.config.id().into(),
            integration: d.config.kind().into(),
            workspace_id: selected.clone(),
            state: status.map_or_else(
                || {
                    if latest.is_some() && !outdated {
                        CollectorState::Ready
                    } else {
                        CollectorState::Waiting
                    }
                },
                |s| s.state,
            ),
            message: status.and_then(|s| s.message.clone()),
            updated_at: status
                .map(|s| s.updated_at)
                .or_else(|| latest.as_ref().map(|v| v.observed_at)),
            freshness_seconds: freshness,
            stale,
            outdated,
            latest,
        });
    }
    Ok(result)
}
fn upstream_outdated(
    declaration: &CollectorDeclaration,
    result: &EvidenceObservation,
    observations: &[EvidenceObservation],
    declarations: &[CollectorDeclaration],
    workspace: &str,
) -> bool {
    use crate::{collectors::CollectorConfig, evidence::EvidenceResult};
    let hash = match &result.result {
        EvidenceResult::License(v) => &v.inventory_hash,
        EvidenceResult::Vulnerability(v) => &v.inventory_hash,
        _ => return false,
    };
    if let CollectorConfig::GrantLicense(config) = &declaration.config {
        let expected = crate::collectors::fingerprint(&config.policy).ok();
        if expected.as_ref() != result.policy_revision.as_ref()
            || matches!(&result.result,EvidenceResult::License(v) if Some(&v.policy_revision)!=expected.as_ref())
        {
            return true;
        }
    }
    let reference = declaration.config.sbom();
    let manifest = reference
        .and_then(|r| r.manifest_id.as_ref())
        .unwrap_or(&declaration.manifest_id);
    let collector = reference.map(|r| r.collector_id.as_str());
    let inventory = observations
        .iter()
        .filter(|v| {
            v.manifest_id == *manifest
                && v.workspace_id == workspace
                && v.environment == result.environment
                && v.integration == "syft_inventory"
                && collector.is_none_or(|id| v.collector_id == id)
        })
        .max_by_key(|v| {
            let current = declarations
                .iter()
                .find(|d| d.manifest_id == v.manifest_id && d.config.id() == v.collector_id)
                .is_none_or(|d| d.revision == v.collector_revision);
            (current, v.observed_at, v.attempt, &v.observation_id)
        });
    let Some(inventory) = inventory else {
        return true;
    };
    let EvidenceResult::Inventory(inventory) = &inventory.result else {
        return true;
    };
    hash != &inventory.artifact_hash
        || result
            .upstream_fingerprint
            .as_ref()
            .is_some_and(|h| h != &inventory.artifact_hash)
}

fn deserialize<T: serde::de::DeserializeOwned>(rows: Vec<String>) -> Result<Vec<T>, String> {
    rows.into_iter()
        .map(|v| serde_json::from_str(&v).map_err(|e| e.to_string()))
        .collect()
}
/// Shared parameterized text query for schema and evidence persistence.
pub(super) async fn query(
    pool: &DatabasePool,
    sql: &str,
    params: Vec<String>,
) -> Result<Vec<String>, String> {
    match pool {
        DatabasePool::Sqlite(p) => {
            // SQL is assembled only from static clauses and placeholder counts; all values are bound.
            let mut query = sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()));
            for v in &params {
                query = query.bind(v);
            }
            query.fetch_all(p).await.map_err(|e| e.to_string())
        }
        DatabasePool::Turso(db) => {
            let c = db.connect().map_err(|e| e.to_string())?;
            let mut rows = c
                .query(
                    sql,
                    params
                        .into_iter()
                        .map(libsql::Value::Text)
                        .collect::<Vec<_>>(),
                )
                .await
                .map_err(|e| e.to_string())?;
            let mut out = vec![];
            while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
                out.push(row.get(0).map_err(|e| e.to_string())?);
            }
            Ok(out)
        }
    }
}
pub(super) async fn execute(
    pool: &DatabasePool,
    sql: &str,
    params: Vec<String>,
) -> Result<u64, String> {
    match pool {
        DatabasePool::Sqlite(p) => {
            let mut q = sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()));
            for v in &params {
                q = q.bind(v);
            }
            q.execute(p)
                .await
                .map(|r| r.rows_affected())
                .map_err(|e| e.to_string())
        }
        DatabasePool::Turso(db) => db
            .connect()
            .map_err(|e| e.to_string())?
            .execute(
                sql,
                params
                    .into_iter()
                    .map(libsql::Value::Text)
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(|e| e.to_string()),
    }
}

#[cfg(test)]
mod tests;
