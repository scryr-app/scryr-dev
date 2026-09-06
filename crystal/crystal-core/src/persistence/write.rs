//! Write-side cleanup helpers for generated manifest persistence.

use super::models::DatabasePool;

async fn prune_legacy_value_artifacts(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<(), String> {
    match pool {
        DatabasePool::Turso(database) => {
            let connection = database
                .connect()
                .map_err(|error| format!("Failed to open Turso cleanup connection: {error}"))?;
            connection
                .execute(
                    r"
                    DELETE FROM generated_manifests
                    WHERE artifact_kind = 'value'
                      AND clerk_org_id = ?
                      AND (
                          manifeset_file_name = ''
                          OR manifeset_file_name = 'manifest.json'
                          OR manifeset_file_name LIKE '%/manifest.json'
                      )
                      AND EXISTS (
                          SELECT 1
                          FROM generated_manifests modern
                          WHERE modern.artifact_kind = 'value'
                            AND modern.clerk_org_id = ?
                            AND modern.manifeset_file_name <> ''
                            AND modern.manifeset_file_name <> 'manifest.json'
                            AND modern.manifeset_file_name NOT LIKE '%/manifest.json'
                      )
                    ",
                    libsql::params![clerk_org_id, clerk_org_id],
                )
                .await
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
        DatabasePool::Sqlite(pool) => sqlx::query(
            r"
                DELETE FROM generated_manifests
                WHERE artifact_kind = 'value'
                  AND clerk_org_id = ?
                  AND (
                      manifeset_file_name = ''
                      OR manifeset_file_name = 'manifest.json'
                      OR manifeset_file_name LIKE '%/manifest.json'
                  )
                  AND EXISTS (
                      SELECT 1
                      FROM generated_manifests modern
                      WHERE modern.artifact_kind = 'value'
                        AND modern.clerk_org_id = ?
                        AND modern.manifeset_file_name <> ''
                        AND modern.manifeset_file_name <> 'manifest.json'
                        AND modern.manifeset_file_name NOT LIKE '%/manifest.json'
                  )
                ",
        )
        .bind(clerk_org_id)
        .bind(clerk_org_id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string()),
    }
    .map_err(|e| format!("Failed to prune legacy generated manifest artifacts: {e}"))?;
    Ok(())
}

/// Remove legacy value-artifact keys after a successful modern value write.
///
/// # Errors
///
/// Returns an error if legacy artifact cleanup fails.
pub(crate) async fn cleanup_legacy_value_artifacts(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<(), String> {
    prune_legacy_value_artifacts(pool, clerk_org_id).await
}
