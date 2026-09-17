//! One-time, transactional installation of editable organization samples.

use super::{
    DatabasePool, metadata::ManifestUploadMetadata, schema, sqlite_write, turso_write,
    write_rows::ManifestWriteRows,
};
use crate::manifest::{ManifestRequestContext, UpsertGeneratedManifestInput};

/// A durable marker also preserves deliberate deletion of starter diagrams.
const SEEDED: &str = "SELECT 1 FROM organization_sample_seeds WHERE clerk_org_id = ?";
/// Claim and sample writes belong to the same transaction; failures remain retryable.
const CLAIM: &str =
    "INSERT INTO organization_sample_seeds (clerk_org_id) VALUES (?) ON CONFLICT DO NOTHING";
/// Snapshot existing sources before inserting any sample siblings.
const EXISTING: &str = "SELECT scry_identifier, folder_path, file_name FROM generated_manifests WHERE clerk_org_id = ? AND artifact_kind IN ('value', 'superseded')";

/// Seed trusted, bundled sample artifacts once for an authenticated organization.
///
/// This is a system initialization operation, including for read-only members.
/// Normal editor authorization continues to govern subsequent changes. Existing
/// sources with conflicting identifiers or locations are preserved in full.
///
/// # Errors
/// Returns storage errors; the marker and all sample writes roll back together.
pub async fn seed_organization_samples(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
    samples: &[UpsertGeneratedManifestInput],
) -> Result<(), String> {
    if context.clerk_org_id.trim().is_empty() || samples.is_empty() {
        return Err("Sample seeding requires an organization and a nonempty catalog".into());
    }
    schema::ensure_table(pool).await?;
    let mut system_context = context.clone();
    system_context.clerk_user_id = "scryr-samples".into();
    match pool {
        DatabasePool::Sqlite(pool) => seed_sqlite(pool, &system_context, samples).await,
        DatabasePool::Turso(database) => seed_turso(database, &system_context, samples).await,
    }
}

/// Skip the whole sample source if any of its diagrams would collide with user work.
fn available_samples<'a>(
    samples: &'a [UpsertGeneratedManifestInput],
    existing: &[(String, String, String)],
) -> Vec<&'a UpsertGeneratedManifestInput> {
    let conflicts: Vec<_> = samples
        .iter()
        .filter(|sample| {
            existing.iter().any(|(identifier, folder, file)| {
                sample.scry_identifier.as_ref() == Some(identifier)
                    || (sample.folder_path.as_ref() == Some(folder)
                        && sample.file_name.as_ref() == Some(file))
            })
        })
        .map(|sample| (&sample.folder_path, &sample.file_name))
        .collect();
    samples
        .iter()
        .filter(|sample| !conflicts.contains(&(&sample.folder_path, &sample.file_name)))
        .collect()
}

/// `SQLite` takes the writer lock at the claim, before inspecting existing sources.
async fn seed_sqlite(
    pool: &sqlx::SqlitePool,
    context: &ManifestRequestContext,
    samples: &[UpsertGeneratedManifestInput],
) -> Result<(), String> {
    if sqlx::query(SEEDED)
        .bind(&context.clerk_org_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(());
    }
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let claimed = sqlx::query(CLAIM)
        .bind(&context.clerk_org_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();
    if claimed != 0 {
        let existing = sqlx::query_as::<_, (String, String, String)>(EXISTING)
            .bind(&context.clerk_org_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        for sample in available_samples(samples, &existing) {
            let meta = ManifestUploadMetadata::from_input(sample);
            let rows = ManifestWriteRows::build(sample, &meta, context);
            let id = sqlite_write::upsert_current_manifest(&mut tx, &rows.current, &meta).await?;
            sqlite_write::insert_upload_ledger_row(&mut tx, &id, &rows.upload).await?;
        }
    }
    tx.commit().await.map_err(|e| e.to_string())
}

/// Turso's immediate transaction serializes initialization across server instances.
async fn seed_turso(
    database: &libsql::Database,
    context: &ManifestRequestContext,
    samples: &[UpsertGeneratedManifestInput],
) -> Result<(), String> {
    let connection = database.connect().map_err(|e| e.to_string())?;
    if connection
        .query(SEEDED, [context.clerk_org_id.clone()])
        .await
        .map_err(|e| e.to_string())?
        .next()
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(());
    }
    let tx = connection
        .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
        .await
        .map_err(|e| e.to_string())?;
    let result = seed_turso_transaction(&tx, context, samples).await;
    if let Err(error) = result {
        tx.rollback().await.map_err(|e| e.to_string())?;
        return Err(error);
    }
    tx.commit().await.map_err(|e| e.to_string())
}

/// Insert the claim, artifacts and upload history on one remote transaction.
async fn seed_turso_transaction(
    tx: &libsql::Transaction,
    context: &ManifestRequestContext,
    samples: &[UpsertGeneratedManifestInput],
) -> Result<(), String> {
    if tx
        .execute(CLAIM, [context.clerk_org_id.clone()])
        .await
        .map_err(|e| e.to_string())?
        == 0
    {
        return Ok(());
    }
    let mut query = tx
        .query(EXISTING, [context.clerk_org_id.clone()])
        .await
        .map_err(|e| e.to_string())?;
    let mut existing = Vec::new();
    while let Some(row) = query.next().await.map_err(|e| e.to_string())? {
        existing.push((
            row.get(0).map_err(|e| e.to_string())?,
            row.get(1).map_err(|e| e.to_string())?,
            row.get(2).map_err(|e| e.to_string())?,
        ));
    }
    drop(query);
    for sample in available_samples(samples, &existing) {
        let meta = ManifestUploadMetadata::from_input(sample);
        let rows = ManifestWriteRows::build(sample, &meta, context);
        let id = turso_write::upsert_current_manifest(tx, &rows.current, &meta).await?;
        turso_write::insert_upload_ledger_row(tx, &id, &rows.upload).await?;
    }
    Ok(())
}
