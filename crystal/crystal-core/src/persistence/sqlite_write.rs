//! Local `SQLite` write paths for generated manifest persistence.

use super::metadata::ManifestUploadMetadata;
use super::sql::{
    INSERT_UPLOAD_LEDGER_SQL, UPSERT_BY_ARTIFACT_KEY_SQL, UPSERT_BY_SCRY_IDENTIFIER_SQL,
};
use super::write_rows::{GeneratedManifestRow, GeneratedManifestUploadRow, ManifestWriteRows};
use crate::manifest::{ManifestRequestContext, UpsertGeneratedManifestInput};

/// Insert or update the current manifest row in `SQLite` and append a ledger row.
pub(super) async fn upsert_generated_manifest_row(
    pool: &sqlx::SqlitePool,
    input: &UpsertGeneratedManifestInput,
    metadata: &ManifestUploadMetadata,
    request_context: &ManifestRequestContext,
) -> Result<String, String> {
    let rows = ManifestWriteRows::build(input, metadata, request_context);
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| format!("Failed to begin SQLite generated manifest upsert: {error}"))?;
    let current_id = upsert_current_manifest(&mut transaction, &rows.current, metadata).await?;

    insert_upload_ledger_row(&mut transaction, &current_id, &rows.upload).await?;

    transaction
        .commit()
        .await
        .map_err(|error| format!("Failed to commit SQLite generated manifest upsert: {error}"))?;

    Ok(current_id)
}

/// Insert or update the current `SQLite` manifest row.
pub(super) async fn upsert_current_manifest(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: &GeneratedManifestRow,
    metadata: &ManifestUploadMetadata,
) -> Result<String, String> {
    let sql = if metadata.scry_identifier.is_empty() {
        UPSERT_BY_ARTIFACT_KEY_SQL
    } else {
        UPSERT_BY_SCRY_IDENTIFIER_SQL
    };

    sqlx::query_scalar::<_, String>(sql)
        .bind(&row.id)
        .bind(&row.common.artifact_kind)
        .bind(&row.common.artifact_key)
        .bind(&row.common.clerk_org_id)
        .bind(&row.common.org_slug)
        .bind(&row.common.uploaded_by_clerk_user_id)
        .bind(&row.common.folder_path)
        .bind(&row.common.file_name)
        .bind(&row.common.scry_identifier)
        .bind(&row.common.name)
        .bind(&row.common.git_commit_sha)
        .bind(&row.common.content)
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| {
            format!(
                "Failed to persist generated artifact {} in SQLite: {error}",
                row.common.artifact_key
            )
        })
}

/// Append a generated manifest upload ledger row in `SQLite`.
pub(super) async fn insert_upload_ledger_row(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    current_id: &str,
    row: &GeneratedManifestUploadRow,
) -> Result<(), String> {
    sqlx::query(INSERT_UPLOAD_LEDGER_SQL)
        .bind(&row.id)
        .bind(current_id)
        .bind(&row.common.artifact_kind)
        .bind(&row.common.artifact_key)
        .bind(&row.common.clerk_org_id)
        .bind(&row.common.org_slug)
        .bind(&row.common.uploaded_by_clerk_user_id)
        .bind(&row.common.folder_path)
        .bind(&row.common.file_name)
        .bind(&row.common.scry_identifier)
        .bind(&row.common.name)
        .bind(&row.common.git_commit_sha)
        .bind(&row.common.content)
        .execute(&mut **transaction)
        .await
        .map(|_| ())
        .map_err(|error| {
            format!(
                "Failed to append generated artifact upload ledger row for {} in SQLite: {error}",
                row.common.artifact_key
            )
        })
}
