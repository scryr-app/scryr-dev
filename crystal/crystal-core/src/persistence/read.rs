//! Read paths for generated manifest persistence.

use super::manifest_envelope::{flatten_manifest_artifact_rows, parse_manifest_blocks_value};
use super::models::{DatabasePool, GeneratedManifestMap, GeneratedManifestMapRow};
use super::read_store;
use super::schema::ensure_table;
use serde_json::Value;

/// Read a generated manifest JSON artifact from storage.
///
/// # Errors
///
/// Returns an error if the storage table is unavailable, the artifact is
/// missing, or the stored content is not valid JSON.
pub(crate) async fn read_generated_manifest_json(
    pool: &DatabasePool,
    clerk_org_id: &str,
    sample: Option<&str>,
) -> Result<Value, String> {
    ensure_table(pool).await?;

    if let Some(sample_key) = sample {
        return read_generated_manifest_json_for_sample(pool, clerk_org_id, sample_key).await;
    }

    read_all_generated_manifest_json(pool, clerk_org_id).await
}

/// List Scryr map value artifacts stored in storage.
///
/// # Errors
///
/// Returns an error if the storage table is unavailable or the rows cannot be queried.
pub(crate) async fn list_generated_manifest_maps(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<Vec<GeneratedManifestMap>, String> {
    ensure_table(pool).await?;

    let rows = read_store::list_maps(pool, clerk_org_id)
        .await
        .map_err(|error| format!("Failed to list generated manifest artifacts: {error}"))?;
    Ok(rows_to_generated_manifest_maps(rows))
}

/// Convert generated manifest map query rows into API structs.
fn rows_to_generated_manifest_maps(
    rows: Vec<GeneratedManifestMapRow>,
) -> Vec<GeneratedManifestMap> {
    rows.into_iter()
        .map(
            |(
                id,
                key,
                scry_identifier,
                name,
                clerk_org_id,
                org_slug,
                folder_path,
                file_name,
                git_commit_sha,
                updated_at,
            )| GeneratedManifestMap {
                id,
                key,
                scry_identifier,
                name,
                clerk_org_id,
                org_slug,
                folder_path,
                file_name,
                git_commit_sha,
                updated_at,
            },
        )
        .collect()
}

/// Read a generated manifest JSON artifact by exact Scryr identifier or row id.
///
/// # Errors
///
/// Returns an error if the storage table is unavailable, the artifact is
/// missing, or the stored content is not valid JSON.
pub(crate) async fn read_generated_manifest_json_by_scry_identifier(
    pool: &DatabasePool,
    clerk_org_id: &str,
    scry_identifier: &str,
) -> Result<Value, String> {
    ensure_table(pool).await?;

    let row = read_store::read_by_scry_identifier(pool, clerk_org_id, scry_identifier)
        .await
        .map_err(|error| {
            format!(
                "Failed to load generated manifest artifact for Scryr identifier {scry_identifier}: {error}"
            )
        })?;
    let Some((artifact_key, content)) = row else {
        return Err(format!(
            "Generated manifest artifact not found for Scryr identifier: {scry_identifier}"
        ));
    };

    parse_manifest_blocks_value(&artifact_key, &content)
}

/// Read a generated manifest JSON artifact for one sample key.
async fn read_generated_manifest_json_for_sample(
    pool: &DatabasePool,
    clerk_org_id: &str,
    sample_key: &str,
) -> Result<Value, String> {
    let legacy_sample_key = format!("{sample_key}/manifest.json");
    let row = read_store::read_sample(pool, clerk_org_id, sample_key, &legacy_sample_key)
        .await
        .map_err(|error| {
            format!("Failed to load generated manifest artifact for sample {sample_key}: {error}")
        })?;
    let Some((artifact_key, content)) = row else {
        return Err(format!(
            "Generated manifest artifact not found for sample: {sample_key}"
        ));
    };

    parse_manifest_blocks_value(&artifact_key, &content)
}

/// Read and combine all generated manifest JSON artifacts.
async fn read_all_generated_manifest_json(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<Value, String> {
    let rows = read_store::read_all_value_artifacts(pool, clerk_org_id)
        .await
        .map_err(|error| format!("Failed to load generated manifest artifacts: {error}"))?;

    if rows.is_empty() {
        return Err("Generated manifest artifacts not found".to_string());
    }

    let modern_rows = rows
        .iter()
        .filter(|(artifact_key, _)| is_modern_manifest_artifact_key(artifact_key))
        .collect::<Vec<_>>();

    if !modern_rows.is_empty() {
        return flatten_manifest_artifact_rows(&modern_rows);
    }

    let legacy_sample_rows = rows
        .iter()
        .filter(|(artifact_key, _)| artifact_key.ends_with("/manifest.json"))
        .collect::<Vec<_>>();

    if !legacy_sample_rows.is_empty() {
        return flatten_manifest_artifact_rows(&legacy_sample_rows);
    }

    let aggregate_row = rows
        .iter()
        .find(|(artifact_key, _)| artifact_key.is_empty() || artifact_key == "manifest.json")
        .ok_or_else(|| "Generated manifest artifacts not found".to_string())?;

    parse_manifest_blocks_value(&aggregate_row.0, &aggregate_row.1)
}

/// Return whether an artifact key uses the new sample-key storage format.
pub(super) fn is_modern_manifest_artifact_key(artifact_key: &str) -> bool {
    !artifact_key.is_empty()
        && artifact_key != "manifest.json"
        && !artifact_key.ends_with("/manifest.json")
}
