//! Backend-specific query helpers for generated manifest reads.

use super::models::{ArtifactKind, DatabasePool, GeneratedManifestMapRow};

const LIST_MAPS_SQL: &str = r"
    SELECT
        COALESCE(NULLIF(scry_identifier, ''), NULLIF(manifeset_file_name, ''), id),
        manifeset_file_name,
        scry_identifier,
        name,
        clerk_org_id,
        org_slug,
        folder_path,
        file_name,
        git_commit_sha,
        updated_at
    FROM generated_manifests
    WHERE artifact_kind = ?
      AND clerk_org_id = ?
    ORDER BY folder_path ASC, file_name ASC, name ASC, updated_at DESC
";

const READ_BY_SCRY_IDENTIFIER_SQL: &str = r"
    SELECT manifeset_file_name, content
    FROM generated_manifests
    WHERE artifact_kind = ?
      AND clerk_org_id = ?
      AND (
          scry_identifier = ?
          OR manifeset_file_name = ?
          OR id = ?
      )
    ORDER BY CASE
        WHEN scry_identifier = ? THEN 0
        WHEN manifeset_file_name = ? THEN 1
        ELSE 2
    END
    LIMIT 1
";

const READ_SAMPLE_SQL: &str = r"
    SELECT manifeset_file_name, content
    FROM generated_manifests
    WHERE artifact_kind = ?
      AND clerk_org_id IN (?, '')
      AND manifeset_file_name IN (?, ?)
    ORDER BY
        CASE WHEN clerk_org_id = ? THEN 0 ELSE 1 END,
        CASE WHEN manifeset_file_name = ? THEN 0 ELSE 1 END
    LIMIT 1
";

const READ_ALL_VALUE_ARTIFACTS_SQL: &str = r"
    SELECT manifeset_file_name, content
    FROM generated_manifests
    WHERE artifact_kind = ?
      AND clerk_org_id = ?
    ORDER BY manifeset_file_name ASC, updated_at DESC
";

/// List generated map rows from the configured storage backend.
pub(super) async fn list_maps(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<Vec<GeneratedManifestMapRow>, String> {
    match pool {
        DatabasePool::Turso(database) => query_turso_map_rows(database, clerk_org_id).await,
        DatabasePool::Sqlite(pool) => sqlx::query_as::<_, GeneratedManifestMapRow>(LIST_MAPS_SQL)
            .bind(ArtifactKind::Value.as_str())
            .bind(clerk_org_id)
            .fetch_all(pool)
            .await
            .map_err(|error| error.to_string()),
    }
}

/// Read a generated manifest row by exact Scryr identifier, artifact key, or row id.
pub(super) async fn read_by_scry_identifier(
    pool: &DatabasePool,
    clerk_org_id: &str,
    scry_identifier: &str,
) -> Result<Option<(String, String)>, String> {
    match pool {
        DatabasePool::Turso(database) => {
            query_turso_optional_manifest_row(
                database,
                READ_BY_SCRY_IDENTIFIER_SQL,
                libsql::params![
                    ArtifactKind::Value.as_str(),
                    clerk_org_id,
                    scry_identifier,
                    scry_identifier,
                    scry_identifier,
                    scry_identifier,
                    scry_identifier
                ],
            )
            .await
        }
        DatabasePool::Sqlite(pool) => {
            sqlx::query_as::<_, (String, String)>(READ_BY_SCRY_IDENTIFIER_SQL)
                .bind(ArtifactKind::Value.as_str())
                .bind(clerk_org_id)
                .bind(scry_identifier)
                .bind(scry_identifier)
                .bind(scry_identifier)
                .bind(scry_identifier)
                .bind(scry_identifier)
                .fetch_optional(pool)
                .await
                .map_err(|error| error.to_string())
        }
    }
}

/// Read a generated manifest row for one sample key.
pub(super) async fn read_sample(
    pool: &DatabasePool,
    clerk_org_id: &str,
    sample_key: &str,
    legacy_sample_key: &str,
) -> Result<Option<(String, String)>, String> {
    match pool {
        DatabasePool::Turso(database) => {
            query_turso_optional_manifest_row(
                database,
                READ_SAMPLE_SQL,
                libsql::params![
                    ArtifactKind::Value.as_str(),
                    clerk_org_id,
                    sample_key,
                    legacy_sample_key,
                    clerk_org_id,
                    sample_key
                ],
            )
            .await
        }
        DatabasePool::Sqlite(pool) => sqlx::query_as::<_, (String, String)>(READ_SAMPLE_SQL)
            .bind(ArtifactKind::Value.as_str())
            .bind(clerk_org_id)
            .bind(sample_key)
            .bind(legacy_sample_key)
            .bind(clerk_org_id)
            .bind(sample_key)
            .fetch_optional(pool)
            .await
            .map_err(|error| error.to_string()),
    }
}

/// Read all generated manifest value artifacts for one organization.
pub(super) async fn read_all_value_artifacts(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<Vec<(String, String)>, String> {
    match pool {
        DatabasePool::Turso(database) => {
            query_turso_manifest_rows(
                database,
                READ_ALL_VALUE_ARTIFACTS_SQL,
                libsql::params![ArtifactKind::Value.as_str(), clerk_org_id],
            )
            .await
        }
        DatabasePool::Sqlite(pool) => {
            sqlx::query_as::<_, (String, String)>(READ_ALL_VALUE_ARTIFACTS_SQL)
                .bind(ArtifactKind::Value.as_str())
                .bind(clerk_org_id)
                .fetch_all(pool)
                .await
                .map_err(|error| error.to_string())
        }
    }
}

/// Query generated map rows from Turso/libSQL storage.
async fn query_turso_map_rows(
    database: &libsql::Database,
    clerk_org_id: &str,
) -> Result<Vec<GeneratedManifestMapRow>, String> {
    let connection = turso_connection(database)?;
    let mut rows = connection
        .query(
            LIST_MAPS_SQL,
            libsql::params![ArtifactKind::Value.as_str(), clerk_org_id],
        )
        .await
        .map_err(|error| format!("Failed to query Turso generated manifest maps: {error}"))?;
    let mut maps = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("Failed to read Turso generated manifest map row: {error}"))?
    {
        maps.push(turso_manifest_map_row(&row)?);
    }

    Ok(maps)
}

/// Open a Turso/libSQL connection.
fn turso_connection(database: &libsql::Database) -> Result<libsql::Connection, String> {
    database
        .connect()
        .map_err(|error| format!("Failed to open Turso connection: {error}"))
}

/// Query at most one manifest artifact row from Turso/libSQL storage.
async fn query_turso_optional_manifest_row(
    database: &libsql::Database,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> Result<Option<(String, String)>, String> {
    let rows = query_turso_manifest_rows(database, sql, params).await?;
    Ok(rows.into_iter().next())
}

/// Query manifest artifact rows from Turso/libSQL storage.
async fn query_turso_manifest_rows(
    database: &libsql::Database,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> Result<Vec<(String, String)>, String> {
    let connection = turso_connection(database)?;
    let mut rows = connection
        .query(sql, params)
        .await
        .map_err(|error| format!("Failed to query Turso generated manifest artifacts: {error}"))?;
    let mut artifacts = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("Failed to read Turso generated manifest artifact row: {error}"))?
    {
        artifacts.push(turso_manifest_artifact_row(&row)?);
    }

    Ok(artifacts)
}

/// Convert a Turso/libSQL row into generated manifest map row metadata.
fn turso_manifest_map_row(row: &libsql::Row) -> Result<GeneratedManifestMapRow, String> {
    Ok((
        row.get::<String>(0)
            .map_err(|error| format!("Failed to read Turso map id: {error}"))?,
        row.get::<String>(1)
            .map_err(|error| format!("Failed to read Turso artifact key: {error}"))?,
        row.get::<String>(2)
            .map_err(|error| format!("Failed to read Turso Scryr identifier: {error}"))?,
        row.get::<String>(3)
            .map_err(|error| format!("Failed to read Turso map name: {error}"))?,
        row.get::<String>(4)
            .map_err(|error| format!("Failed to read Turso organization id: {error}"))?,
        row.get::<Option<String>>(5)
            .map_err(|error| format!("Failed to read Turso organization slug: {error}"))?,
        row.get::<String>(6)
            .map_err(|error| format!("Failed to read Turso folder path: {error}"))?,
        row.get::<String>(7)
            .map_err(|error| format!("Failed to read Turso file name: {error}"))?,
        row.get::<Option<String>>(8)
            .map_err(|error| format!("Failed to read Turso git commit SHA: {error}"))?,
        row.get::<String>(9)
            .map_err(|error| format!("Failed to read Turso update timestamp: {error}"))?,
    ))
}

/// Convert a Turso/libSQL row into a manifest artifact key/content pair.
fn turso_manifest_artifact_row(row: &libsql::Row) -> Result<(String, String), String> {
    Ok((
        row.get::<String>(0)
            .map_err(|error| format!("Failed to read Turso artifact key: {error}"))?,
        row.get::<String>(1)
            .map_err(|error| format!("Failed to read Turso artifact content: {error}"))?,
    ))
}
