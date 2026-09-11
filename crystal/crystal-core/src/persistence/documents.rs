//! Source snapshots and transactional diagram replacement, shared by SQLite and Turso.
use super::{
    DatabasePool, metadata::ManifestUploadMetadata, read_store, schema, sqlite_write, turso_write,
    write_rows::ManifestWriteRows,
};
use crate::{
    generated_manifest_envelope::ManifestSourceFile,
    manifest::{ArtifactKind, ManifestRequestContext, UpsertGeneratedManifestInput},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Source exposed to an editor. Revisions cover every diagram from the same entrypoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestDocument {
    /// Selected diagram identifier.
    pub identifier: String,
    /// Persisted artifact key.
    pub key: String,
    /// Display-only source folder; never accepted as a local filesystem path.
    pub folder_path: String,
    /// Relative entrypoint within files.
    pub entrypoint: String,
    /// Source tree captured by the CLI.
    pub files: Vec<ManifestSourceFile>,
    /// Opaque optimistic concurrency token.
    pub revision: String,
    /// Whether this principal can save.
    pub writable: bool,
    /// Whether saving also writes the registered local entrypoint.
    pub local: bool,
}

/// Hash a serialized snapshot.
#[must_use]
pub fn revision(value: &impl Serialize) -> String {
    use std::fmt::Write;
    Sha256::digest(serde_json::to_vec(value).unwrap_or_default())
        .iter()
        .fold(String::new(), |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        })
}

/// Stored value rows, including source location and contents.
pub type DocumentRow = (String, String, String, String, String);
pub(crate) const ROWS_SQL: &str = "SELECT manifeset_file_name, scry_identifier, folder_path, file_name, content FROM generated_manifests WHERE artifact_kind = 'value' AND clerk_org_id = ? ORDER BY manifeset_file_name, scry_identifier";
const ARCHIVE_SQL: &str = "UPDATE generated_manifests SET artifact_kind = 'superseded' WHERE artifact_kind = 'value' AND clerk_org_id = ? AND folder_path = ? AND file_name = ? AND scry_identifier = ?";

/// Read the source of an existing diagram within a single tenant.
///
/// # Errors
/// Returns an error when the diagram or its source is unavailable.
pub async fn read_document(
    pool: &DatabasePool,
    org: &str,
    identifier: &str,
) -> Result<ManifestDocument, String> {
    schema::ensure_table(pool).await?;
    let maps = super::read::list_generated_manifest_maps(pool, org).await?;
    let selected = maps
        .iter()
        .find(|m| m.scry_identifier == identifier || m.id == identifier || m.key == identifier)
        .ok_or("Selected diagram no longer exists")?;
    let mut rows = Vec::new();
    let mut files = None;
    for map in maps
        .iter()
        .filter(|m| m.folder_path == selected.folder_path && m.file_name == selected.file_name)
    {
        let (_, content) = read_store::read_by_scry_identifier(pool, org, &map.id)
            .await?
            .ok_or("Diagram changed while reading source; reload")?;
        if map.id == selected.id {
            let value: serde_json::Value =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;
            files = Some(
                serde_json::from_value::<Vec<ManifestSourceFile>>(value["files"].clone())
                    .map_err(|e| e.to_string())?,
            );
        }
        rows.push((
            map.key.clone(),
            map.scry_identifier.clone(),
            map.folder_path.clone(),
            map.file_name.clone(),
            content,
        ));
    }
    rows.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    let files = files.ok_or("Manifest source is unavailable")?;
    if !files.iter().any(|f| f.path == selected.file_name) {
        return Err("Manifest entrypoint is missing from its source snapshot".into());
    }
    Ok(ManifestDocument {
        identifier: selected.scry_identifier.clone(),
        key: selected.key.clone(),
        folder_path: selected.folder_path.clone(),
        entrypoint: selected.file_name.clone(),
        files,
        revision: revision(&(org, &rows)),
        writable: false,
        local: false,
    })
}

fn verify_snapshot(
    rows: &[DocumentRow],
    doc: &ManifestDocument,
    inputs: &[UpsertGeneratedManifestInput],
    org: &str,
) -> Result<(), String> {
    let own: Vec<_> = rows
        .iter()
        .filter(|r| r.2 == doc.folder_path && r.3 == doc.entrypoint)
        .cloned()
        .collect();
    if revision(&(org, &own)) != doc.revision {
        return Err("Source changed since it was loaded. Reload before saving.".into());
    }
    for input in inputs {
        if rows.iter().any(|r| {
            Some(&r.1) == input.scry_identifier.as_ref()
                && (r.2 != doc.folder_path || r.3 != doc.entrypoint)
        }) {
            return Err("A diagram with this identifier belongs to another source file".into());
        }
    }
    Ok(())
}

/// Validate and split a browser-generated document with the same transformation as CLI push.
///
/// # Errors
/// Rejects malformed envelopes or edits outside the entrypoint.
pub fn prepare_save(
    doc: &ManifestDocument,
    envelope: &serde_json::Value,
) -> Result<Vec<UpsertGeneratedManifestInput>, String> {
    let files: Vec<ManifestSourceFile> =
        serde_json::from_value(envelope["files"].clone()).map_err(|e| e.to_string())?;
    if files.len() != doc.files.len()
        || files.iter().zip(&doc.files).any(|(a, b)| {
            let is_entrypoint = a.path == doc.entrypoint;
            let supporting_file_changed = !is_entrypoint && a.content != b.content;
            a.path != b.path || supporting_file_changed
        })
    {
        return Err(
            "Only the selected entrypoint can be edited; supporting files must be preserved".into(),
        );
    }
    if envelope["diagrams"].as_array().is_none_or(Vec::is_empty) {
        return Err("Define at least one public Diagram".into());
    }
    let content = serde_json::to_string(envelope).map_err(|e| e.to_string())?;
    let maps = crate::generation::map_artifacts_from_manifest_json(&content, &doc.key)
        .map_err(|e| e.to_string())?;
    maps.into_iter()
        .map(|map| {
            super::manifest_envelope::parse_manifest_blocks_value(
                &map.artifact_key,
                &map.manifest_json,
            )?;
            Ok(UpsertGeneratedManifestInput {
                artifact_kind: ArtifactKind::Value,
                artifact_key: map.artifact_key,
                scry_identifier: Some(map.map_metadata.scry_identifier),
                name: Some(map.map_metadata.name),
                folder_path: Some(doc.folder_path.clone()),
                file_name: Some(doc.entrypoint.clone()),
                git_commit_sha: None,
                content: map.manifest_json,
            })
        })
        .collect()
}

/// Commit all diagrams and their embedded source together, rejecting stale edits.
///
/// # Errors
/// Returns validation, revision-conflict, or database errors. Transactions roll back on failure.
pub async fn save_document(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    doc: &ManifestDocument,
    inputs: &[UpsertGeneratedManifestInput],
) -> Result<(), String> {
    if !context.can_write_generated_manifests() {
        return Err("Active organization role cannot edit manifests".into());
    }
    match pool {
        DatabasePool::Sqlite(pool) => {
            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            let rows = sqlx::query_as::<_, DocumentRow>(ROWS_SQL)
                .bind(&context.clerk_org_id)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            verify_snapshot(&rows, doc, inputs, &context.clerk_org_id)?;
            for input in inputs {
                let input = preserve_artifact_key(input, &rows);
                let meta = ManifestUploadMetadata::from_input(&input);
                let write = ManifestWriteRows::build(&input, &meta, context);
                let id =
                    sqlite_write::upsert_current_manifest(&mut tx, &write.current, &meta).await?;
                sqlite_write::insert_upload_ledger_row(&mut tx, &id, &write.upload).await?;
            }
            for row in obsolete(&rows, doc, inputs) {
                sqlx::query(ARCHIVE_SQL)
                    .bind(&context.clerk_org_id)
                    .bind(&doc.folder_path)
                    .bind(&doc.entrypoint)
                    .bind(&row.1)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            tx.commit().await.map_err(|e| e.to_string())
        }
        DatabasePool::Turso(database) => {
            let connection = database.connect().map_err(|e| e.to_string())?;
            let tx = connection
                .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
                .await
                .map_err(|e| e.to_string())?;
            let result = save_turso(&tx, context, doc, inputs).await;
            if let Err(error) = result {
                tx.rollback().await.map_err(|e| e.to_string())?;
                return Err(error);
            }
            tx.commit().await.map_err(|e| e.to_string())
        }
    }
}

async fn save_turso(
    tx: &libsql::Transaction,
    context: &ManifestRequestContext,
    doc: &ManifestDocument,
    inputs: &[UpsertGeneratedManifestInput],
) -> Result<(), String> {
    let mut query = tx
        .query(ROWS_SQL, [context.clerk_org_id.clone()])
        .await
        .map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    while let Some(row) = query.next().await.map_err(|e| e.to_string())? {
        rows.push((
            row.get(0).map_err(|e| e.to_string())?,
            row.get(1).map_err(|e| e.to_string())?,
            row.get(2).map_err(|e| e.to_string())?,
            row.get(3).map_err(|e| e.to_string())?,
            row.get(4).map_err(|e| e.to_string())?,
        ));
    }
    drop(query);
    verify_snapshot(&rows, doc, inputs, &context.clerk_org_id)?;
    for input in inputs {
        let input = preserve_artifact_key(input, &rows);
        let meta = ManifestUploadMetadata::from_input(&input);
        let write = ManifestWriteRows::build(&input, &meta, context);
        let id = turso_write::upsert_current_manifest(tx, &write.current, &meta).await?;
        turso_write::insert_upload_ledger_row(tx, &id, &write.upload).await?;
    }
    for row in obsolete(&rows, doc, inputs) {
        tx.execute(
            ARCHIVE_SQL,
            libsql::params![
                context.clerk_org_id.clone(),
                doc.folder_path.clone(),
                doc.entrypoint.clone(),
                row.1.clone()
            ],
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn preserve_artifact_key(
    input: &UpsertGeneratedManifestInput,
    rows: &[DocumentRow],
) -> UpsertGeneratedManifestInput {
    let mut input = input.clone();
    if let Some(row) = rows
        .iter()
        .find(|row| input.scry_identifier.as_ref() == Some(&row.1))
    {
        input.artifact_key.clone_from(&row.0);
    }
    input
}

fn obsolete<'a>(
    rows: &'a [DocumentRow],
    doc: &ManifestDocument,
    inputs: &[UpsertGeneratedManifestInput],
) -> Vec<&'a DocumentRow> {
    rows.iter()
        .filter(|r| {
            r.2 == doc.folder_path
                && r.3 == doc.entrypoint
                && !inputs
                    .iter()
                    .any(|i| i.scry_identifier.as_ref() == Some(&r.1))
        })
        .collect()
}
