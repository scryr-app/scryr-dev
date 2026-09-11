//! Turso/libSQL write paths for generated manifest persistence.

use super::metadata::ManifestUploadMetadata;
use super::sql::{
    INSERT_UPLOAD_LEDGER_SQL, UPSERT_BY_ARTIFACT_KEY_SQL, UPSERT_BY_SCRY_IDENTIFIER_SQL,
};
use super::write_rows::{GeneratedManifestRow, GeneratedManifestUploadRow, ManifestWriteRows};
use crate::manifest::{ManifestRequestContext, UpsertGeneratedManifestInput};

/// Insert or update the current manifest row in Turso/libSQL and append a ledger row.
pub(super) async fn upsert_generated_manifest_row(
    database: &libsql::Database,
    input: &UpsertGeneratedManifestInput,
    metadata: &ManifestUploadMetadata,
    request_context: &ManifestRequestContext,
) -> Result<String, String> {
    let rows = ManifestWriteRows::build(input, metadata, request_context);
    let connection = database
        .connect()
        .map_err(|error| format!("Failed to open Turso generated manifest connection: {error}"))?;
    let transaction = connection
        .transaction()
        .await
        .map_err(|error| format!("Failed to begin Turso generated manifest upsert: {error}"))?;
    let current_id = upsert_current_manifest(&transaction, &rows.current, metadata).await?;

    insert_upload_ledger_row(&transaction, &current_id, &rows.upload).await?;

    transaction
        .commit()
        .await
        .map_err(|error| format!("Failed to commit Turso generated manifest upsert: {error}"))?;

    Ok(current_id)
}

/// Insert or update the current Turso/libSQL manifest row.
pub(super) async fn upsert_current_manifest(
    transaction: &libsql::Transaction,
    row: &GeneratedManifestRow,
    metadata: &ManifestUploadMetadata,
) -> Result<String, String> {
    let sql = if metadata.scry_identifier.is_empty() {
        UPSERT_BY_ARTIFACT_KEY_SQL
    } else {
        UPSERT_BY_SCRY_IDENTIFIER_SQL
    };

    let mut rows = transaction
        .query(
            sql,
            libsql::params![
                row.id.clone(),
                row.common.artifact_kind.clone(),
                row.common.artifact_key.clone(),
                row.common.clerk_org_id.clone(),
                row.common.org_slug.clone(),
                row.common.uploaded_by_clerk_user_id.clone(),
                row.common.folder_path.clone(),
                row.common.file_name.clone(),
                row.common.scry_identifier.clone(),
                row.common.name.clone(),
                row.common.git_commit_sha.clone(),
                row.common.content.clone()
            ],
        )
        .await
        .map_err(|error| {
            format!(
                "Failed to persist generated artifact {} in Turso: {error}",
                row.common.artifact_key
            )
        })?;
    let returned_row = rows
        .next()
        .await
        .map_err(|error| {
            format!(
                "Failed to read persisted generated artifact id for {} from Turso: {error}",
                row.common.artifact_key
            )
        })?
        .ok_or_else(|| {
            format!(
                "Turso did not return a persisted generated artifact id for {}",
                row.common.artifact_key
            )
        })?;

    returned_row.get::<String>(0).map_err(|error| {
        format!(
            "Failed to decode persisted generated artifact id for {} from Turso: {error}",
            row.common.artifact_key
        )
    })
}

/// Append a generated manifest upload ledger row in Turso/libSQL.
pub(super) async fn insert_upload_ledger_row(
    transaction: &libsql::Transaction,
    current_id: &str,
    row: &GeneratedManifestUploadRow,
) -> Result<(), String> {
    transaction
        .execute(
            INSERT_UPLOAD_LEDGER_SQL,
            libsql::params![
                row.id.clone(),
                current_id,
                row.common.artifact_kind.clone(),
                row.common.artifact_key.clone(),
                row.common.clerk_org_id.clone(),
                row.common.org_slug.clone(),
                row.common.uploaded_by_clerk_user_id.clone(),
                row.common.folder_path.clone(),
                row.common.file_name.clone(),
                row.common.scry_identifier.clone(),
                row.common.name.clone(),
                row.common.git_commit_sha.clone(),
                row.common.content.clone()
            ],
        )
        .await
        .map(|_| ())
        .map_err(|error| {
            format!(
                "Failed to append generated artifact upload ledger row for {} in Turso: {error}",
                row.common.artifact_key
            )
        })
}
