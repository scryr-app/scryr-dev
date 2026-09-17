//! Tenant-scoped GitHub dependency snapshots with independent component freshness.

use super::{DatabasePool, schema};
use crate::{github_dependencies::GithubDependencySnapshot, manifest::ManifestRequestContext};
use chrono::SecondsFormat;

pub(super) const CREATE_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS manifest_github_dependencies (
    clerk_org_id TEXT NOT NULL,
    manifest_id TEXT NOT NULL,
    repository TEXT NOT NULL,
    inventory_observed_at TEXT NOT NULL,
    inventory TEXT NOT NULL,
    security_observed_at TEXT NOT NULL,
    security TEXT NOT NULL,
    PRIMARY KEY (clerk_org_id, manifest_id)
)";

// ISO timestamps are normalized to fixed-width UTC nanoseconds before binding.
// Each provided component can advance independently, without replacing the other.
const RECORD_SNAPSHOT: &str = r"
INSERT INTO manifest_github_dependencies (
    clerk_org_id, manifest_id, repository, inventory_observed_at, inventory, security_observed_at, security
) VALUES (?, ?, ?, ?, ?, ?, ?)
ON CONFLICT (clerk_org_id, manifest_id) DO UPDATE SET
    repository = excluded.repository,
    inventory_observed_at = CASE WHEN repository <> excluded.repository OR excluded.inventory_observed_at >= inventory_observed_at
        THEN excluded.inventory_observed_at ELSE inventory_observed_at END,
    inventory = CASE WHEN repository <> excluded.repository OR (excluded.inventory_observed_at <> '' AND excluded.inventory_observed_at >= inventory_observed_at)
        THEN excluded.inventory ELSE inventory END,
    security_observed_at = CASE WHEN repository <> excluded.repository OR excluded.security_observed_at >= security_observed_at
        THEN excluded.security_observed_at ELSE security_observed_at END,
    security = CASE WHEN repository <> excluded.repository OR (excluded.security_observed_at <> '' AND excluded.security_observed_at >= security_observed_at)
        THEN excluded.security ELSE security END
WHERE (repository <> excluded.repository
        AND MAX(excluded.inventory_observed_at, excluded.security_observed_at) >= MAX(inventory_observed_at, security_observed_at))
    OR (repository = excluded.repository AND (
        (excluded.inventory_observed_at <> '' AND excluded.inventory_observed_at >= inventory_observed_at AND excluded.inventory <> inventory)
        OR (excluded.security_observed_at <> '' AND excluded.security_observed_at >= security_observed_at AND excluded.security <> security)
    ))";

const READ_SNAPSHOT: &str = "SELECT repository, inventory, security FROM manifest_github_dependencies WHERE clerk_org_id = ? AND manifest_id = ?";

/// Record complete successful components, preserving newer and omitted components.
///
/// # Errors
/// Rejects unauthorized writes, invalid snapshots, invalid manifest IDs, or storage failures.
pub async fn record_github_dependencies(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    manifest_id: &str,
    mut snapshot: GithubDependencySnapshot,
) -> Result<bool, String> {
    if context.clerk_org_id.is_empty() || !context.can_write_generated_manifests() {
        return Err("active organization role cannot record GitHub dependencies".into());
    }
    crate::reports::validate_manifest_id(manifest_id)?;
    snapshot.validate()?;
    snapshot.repository.make_ascii_lowercase();
    let parameters = [
        context.clerk_org_id.clone(),
        manifest_id.into(),
        snapshot.repository,
        snapshot
            .inventory
            .as_ref()
            .map(|inventory| {
                inventory
                    .observed_at
                    .to_rfc3339_opts(SecondsFormat::Nanos, true)
            })
            .unwrap_or_default(),
        serde_json::to_string(&snapshot.inventory).map_err(|error| error.to_string())?,
        snapshot
            .security
            .as_ref()
            .map(|security| {
                security
                    .observed_at
                    .to_rfc3339_opts(SecondsFormat::Nanos, true)
            })
            .unwrap_or_default(),
        serde_json::to_string(&snapshot.security).map_err(|error| error.to_string())?,
    ];
    schema::ensure_table(pool).await?;
    let affected = match pool {
        DatabasePool::Sqlite(pool) => {
            let mut query = sqlx::query(RECORD_SNAPSHOT);
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
                RECORD_SNAPSHOT,
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

/// Read the latest independently merged snapshot for one tenant and manifest.
///
/// # Errors
/// Rejects missing organizations, invalid manifest IDs, or storage failures.
pub async fn read_github_dependencies(
    pool: &DatabasePool,
    clerk_org_id: &str,
    manifest_id: &str,
) -> Result<Option<GithubDependencySnapshot>, String> {
    if clerk_org_id.is_empty() {
        return Err("GitHub dependencies require an organization".into());
    }
    crate::reports::validate_manifest_id(manifest_id)?;
    schema::ensure_table(pool).await?;
    let row: Option<(String, String, String)> = match pool {
        DatabasePool::Sqlite(pool) => sqlx::query_as(READ_SNAPSHOT)
            .bind(clerk_org_id)
            .bind(manifest_id)
            .fetch_optional(pool)
            .await
            .map_err(|error| error.to_string())?,
        DatabasePool::Turso(database) => {
            let connection = database.connect().map_err(|error| error.to_string())?;
            let mut rows = connection
                .query(READ_SNAPSHOT, [clerk_org_id, manifest_id])
                .await
                .map_err(|error| error.to_string())?;
            rows.next()
                .await
                .map_err(|error| error.to_string())?
                .map(|row| {
                    Ok::<_, String>((
                        row.get(0).map_err(|error| error.to_string())?,
                        row.get(1).map_err(|error| error.to_string())?,
                        row.get(2).map_err(|error| error.to_string())?,
                    ))
                })
                .transpose()?
        }
    };
    row.map(|(repository, inventory, security)| {
        Ok(GithubDependencySnapshot {
            repository,
            inventory: serde_json::from_str(&inventory).map_err(|error| error.to_string())?,
            security: serde_json::from_str(&security).map_err(|error| error.to_string())?,
        })
    })
    .transpose()
}

#[cfg(test)]
mod tests;
