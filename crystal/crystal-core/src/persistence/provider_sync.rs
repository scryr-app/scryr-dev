//! Durable collection health, kept separate from the collected workflow outcomes.

use super::{DatabasePool, schema};
use crate::{manifest::ManifestRequestContext, reports::validate_manifest_id};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) const CREATE_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS manifest_provider_sync (
    clerk_org_id TEXT NOT NULL,
    manifest_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY (clerk_org_id, manifest_id, provider)
)";

const RECORD_SYNC: &str = r"
INSERT INTO manifest_provider_sync (clerk_org_id, manifest_id, provider, content)
VALUES (?, ?, ?, ?)
ON CONFLICT (clerk_org_id, manifest_id, provider) DO UPDATE SET
content = json_set(excluded.content, '$.lastSuccessAt', COALESCE(
    json_extract(excluded.content, '$.lastSuccessAt'),
    json_extract(manifest_provider_sync.content, '$.lastSuccessAt')
), '$.context', json(CASE
    WHEN json_extract(excluded.content, '$.error') IS NOT NULL THEN COALESCE(
        json_extract(excluded.content, '$.context'),
        json_extract(manifest_provider_sync.content, '$.context')
    )
    ELSE json_extract(excluded.content, '$.context')
END))";

const READ_SYNC: &str = r"
SELECT provider, content FROM manifest_provider_sync
WHERE clerk_org_id = ? AND manifest_id = ? ORDER BY provider";

/// Latest collection attempt and last successful collection for a provider.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSyncStatus {
    /// Server receipt time for the latest collection attempt.
    pub last_attempt_at: DateTime<Utc>,
    /// Server receipt time for the latest successful collection, if any.
    pub last_success_at: Option<DateTime<Utc>>,
    /// Collection failure, independent of workflow or job failures.
    pub error: Option<String>,
    /// Provider-specific collection scope, such as a resolved repository default branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

/// Record a completed collection attempt, preserving the previous success on failure.
///
/// # Errors
/// Returns an error for unauthorized writes, invalid identity, or unavailable storage.
pub async fn record_provider_sync(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    manifest_id: &str,
    provider: &str,
    error: Option<String>,
) -> Result<bool, String> {
    record_provider_sync_with_context(pool, context, manifest_id, provider, error, None).await
}

/// Record collection health and an optional provider-specific JSON object.
///
/// Context is limited to 4096 serialized bytes; a failure without context retains
/// the previous context, while a successful collection replaces it.
///
/// # Errors
/// Returns an error for unauthorized writes, invalid identity/context, or unavailable storage.
pub async fn record_provider_sync_with_context(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    manifest_id: &str,
    provider: &str,
    error: Option<String>,
    provider_context: Option<serde_json::Value>,
) -> Result<bool, String> {
    if context.clerk_org_id.is_empty() || !context.can_write_generated_manifests() {
        return Err("active organization role cannot record provider sync".into());
    }
    validate_manifest_id(manifest_id)?;
    if provider.is_empty()
        || provider.len() > 64
        || !provider
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(
            "provider must contain 1 to 64 lowercase letters, digits, or underscores".into(),
        );
    }
    if error.as_ref().is_some_and(|message| message.len() > 4096) {
        return Err("provider sync error must contain at most 4096 bytes".into());
    }
    if let Some(value) = &provider_context {
        if !value.is_object() {
            return Err("provider sync context must be a JSON object".into());
        }
        if serde_json::to_vec(value)
            .map_err(|error| error.to_string())?
            .len()
            > 4096
        {
            return Err("provider sync context must contain at most 4096 bytes".into());
        }
    }
    let now = Utc::now();
    let status = ProviderSyncStatus {
        last_attempt_at: now,
        last_success_at: error.is_none().then_some(now),
        error,
        context: provider_context,
    };
    let parameters = [
        context.clerk_org_id.clone(),
        manifest_id.into(),
        provider.into(),
        serde_json::to_string(&status).map_err(|error| error.to_string())?,
    ];
    schema::ensure_table(pool).await?;
    let affected = match pool {
        DatabasePool::Sqlite(pool) => {
            let mut query = sqlx::query(RECORD_SYNC);
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
                RECORD_SYNC,
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

/// Read collection health for all providers attached to a tenant's stable manifest id.
///
/// # Errors
/// Returns an error for missing organization, invalid identity, or unavailable storage.
pub async fn read_provider_sync(
    pool: &DatabasePool,
    clerk_org_id: &str,
    manifest_id: &str,
) -> Result<BTreeMap<String, ProviderSyncStatus>, String> {
    validate_manifest_id(manifest_id)?;
    if clerk_org_id.is_empty() {
        return Err("provider sync requires an organization".into());
    }
    schema::ensure_table(pool).await?;
    let contents: Vec<(String, String)> = match pool {
        DatabasePool::Sqlite(pool) => sqlx::query_as(READ_SYNC)
            .bind(clerk_org_id)
            .bind(manifest_id)
            .fetch_all(pool)
            .await
            .map_err(|error| error.to_string())?,
        DatabasePool::Turso(database) => {
            let connection = database.connect().map_err(|error| error.to_string())?;
            let mut rows = connection
                .query(READ_SYNC, [clerk_org_id, manifest_id])
                .await
                .map_err(|error| error.to_string())?;
            let mut contents = Vec::new();
            while let Some(row) = rows.next().await.map_err(|error| error.to_string())? {
                contents.push((
                    row.get(0).map_err(|error| error.to_string())?,
                    row.get(1).map_err(|error| error.to_string())?,
                ));
            }
            contents
        }
    };
    contents
        .into_iter()
        .map(|(provider, content)| {
            serde_json::from_str(&content)
                .map(|status| (provider, status))
                .map_err(|error| error.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod context_tests {
    use super::*;

    fn principal() -> ManifestRequestContext {
        ManifestRequestContext {
            clerk_user_id: "user".into(),
            clerk_org_id: "org".into(),
            clerk_org_slug: None,
            clerk_org_role: Some("org:admin".into()),
            clerk_org_permissions: vec![],
        }
    }

    #[tokio::test]
    async fn context_tracks_collection_scope_and_survives_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let pool = DatabasePool::Sqlite(sqlite);
        let principal = principal();
        let initial = serde_json::json!({"repository":"owner/repo", "defaultBranch":"main"});
        record_provider_sync_with_context(
            &pool,
            &principal,
            "api",
            "github",
            None,
            Some(initial.clone()),
        )
        .await?;
        let first = read_provider_sync(&pool, "org", "api").await?;
        assert_eq!(first["github"].context, Some(initial.clone()));
        record_provider_sync(&pool, &principal, "api", "github", Some("offline".into())).await?;
        let failed = read_provider_sync(&pool, "org", "api").await?;
        assert_eq!(failed["github"].context, Some(initial));
        assert_eq!(
            failed["github"].last_success_at,
            first["github"].last_success_at
        );
        let changed = serde_json::json!({"repository":"owner/repo", "defaultBranch":"trunk"});
        record_provider_sync_with_context(
            &pool,
            &principal,
            "api",
            "github",
            Some("jobs unavailable".into()),
            Some(changed.clone()),
        )
        .await?;
        assert_eq!(
            read_provider_sync(&pool, "org", "api").await?["github"].context,
            Some(changed)
        );
        record_provider_sync(&pool, &principal, "api", "github", None).await?;
        assert!(
            read_provider_sync(&pool, "org", "api").await?["github"]
                .context
                .is_none()
        );
        Ok(())
    }

    #[tokio::test]
    async fn context_requires_a_bounded_json_object() -> Result<(), Box<dyn std::error::Error>> {
        let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let pool = DatabasePool::Sqlite(sqlite);
        let principal = principal();
        for value in [
            serde_json::Value::Null,
            serde_json::json!([]),
            serde_json::json!("text"),
            serde_json::json!({"v":"x".repeat(4089)}),
            serde_json::json!({"v":"é".repeat(2045)}),
        ] {
            assert!(
                record_provider_sync_with_context(
                    &pool,
                    &principal,
                    "api",
                    "github",
                    None,
                    Some(value)
                )
                .await
                .is_err()
            );
        }
        let maximum = serde_json::json!({"v":"x".repeat(4088)});
        assert_eq!(serde_json::to_vec(&maximum)?.len(), 4096);
        record_provider_sync_with_context(
            &pool,
            &principal,
            "api",
            "github",
            None,
            Some(maximum.clone()),
        )
        .await?;
        assert_eq!(
            read_provider_sync(&pool, "org", "api").await?["github"].context,
            Some(maximum)
        );
        let legacy: ProviderSyncStatus = serde_json::from_value(serde_json::json!({
            "lastAttemptAt":"2026-09-17T10:00:00Z", "lastSuccessAt":null, "error":"offline"
        }))?;
        assert!(legacy.context.is_none());
        Ok(())
    }
}
