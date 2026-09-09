//! Persistence helpers for generated manifest artifacts.

use crate::manifest::{ManifestRequestContext, UpsertGeneratedManifestInput};
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

mod reports;
pub use reports::{read_reports, record_report};
mod action_history;
pub use action_history::{read_action_history, record_action_run};

mod connection;
mod metadata;
mod persist;
mod sql;
mod sqlite_write;
mod turso_write;
mod write_rows;

mod manifest_envelope;
mod models;
mod read;
mod read_store;
mod schema;
mod write;

pub use connection::sqlite_path_from_env;
pub use models::{ArtifactKind, DatabasePool, GeneratedManifestMap};

/// Persist a generated manifest artifact using the shared SQL layer.
///
/// # Errors
///
/// Returns an error if schema bootstrap, upsert execution, or UUID parsing fails.
pub async fn persist_generated_manifest(
    pool: &DatabasePool,
    input: &UpsertGeneratedManifestInput,
    request_context: &ManifestRequestContext,
) -> Result<Uuid, crate::Error> {
    persist::persist_generated_manifest(pool, input, request_context)
        .await
        .map_err(crate::Error::Storage)
}

/// Connect to storage from environment, defaulting to local `SQLite` when unset.
///
/// # Errors
///
/// Returns an error if the configured database cannot be opened.
pub async fn connect_from_env() -> Result<DatabasePool, crate::Error> {
    connection::connect_from_env()
        .await
        .map_err(crate::Error::Storage)
}

/// Connect to a `SQLite` database file, creating the file and its parent directory.
///
/// # Errors
///
/// Returns an error if the path cannot be prepared or `SQLite` cannot connect.
pub async fn connect_sqlite_path(path: &Path) -> Result<DatabasePool, crate::Error> {
    connection::connect_sqlite_path(path)
        .await
        .map_err(crate::Error::Storage)
}

/// List Scryr map value artifacts stored in storage.
///
/// # Errors
///
/// Returns an error if the storage table is unavailable or the rows cannot be queried.
pub async fn list_generated_manifest_maps(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<Vec<GeneratedManifestMap>, crate::Error> {
    read::list_generated_manifest_maps(pool, clerk_org_id)
        .await
        .map_err(crate::Error::Storage)
}

/// Read a generated manifest JSON artifact from storage.
///
/// # Errors
///
/// Returns an error if the storage table is unavailable, the artifact is
/// missing, or the stored content is not valid JSON.
pub async fn read_generated_manifest_json(
    pool: &DatabasePool,
    clerk_org_id: &str,
    sample: Option<&str>,
) -> Result<Value, crate::Error> {
    read::read_generated_manifest_json(pool, clerk_org_id, sample)
        .await
        .map_err(crate::Error::Storage)
}

/// Read a generated manifest JSON artifact by exact Scryr identifier or row id.
///
/// # Errors
///
/// Returns an error if the storage table is unavailable, the artifact is
/// missing, or the stored content is not valid JSON.
pub async fn read_generated_manifest_json_by_scry_identifier(
    pool: &DatabasePool,
    clerk_org_id: &str,
    scry_identifier: &str,
) -> Result<Value, crate::Error> {
    read::read_generated_manifest_json_by_scry_identifier(pool, clerk_org_id, scry_identifier)
        .await
        .map_err(crate::Error::Storage)
}

/// Create the artifact storage table when it does not already exist.
///
/// # Errors
///
/// Returns an error if any schema bootstrap or migration step fails.
pub async fn ensure_table(pool: &DatabasePool) -> Result<(), crate::Error> {
    schema::ensure_table(pool)
        .await
        .map_err(crate::Error::Storage)
}

/// Remove legacy value-artifact keys after a successful modern value write.
///
/// # Errors
///
/// Returns an error if legacy artifact cleanup fails.
pub async fn cleanup_legacy_value_artifacts(
    pool: &DatabasePool,
    clerk_org_id: &str,
) -> Result<(), crate::Error> {
    write::cleanup_legacy_value_artifacts(pool, clerk_org_id)
        .await
        .map_err(crate::Error::Storage)
}

#[cfg(test)]
mod tests {
    use super::ArtifactKind;
    use super::DatabasePool;
    use super::connect_sqlite_path;
    use super::connection::sqlite_path_from_url;
    use super::ensure_table;
    use super::manifest_envelope::parse_manifest_blocks_value;
    use super::read::is_modern_manifest_artifact_key;
    use super::read_generated_manifest_json;
    use sqlx::Row;

    async fn test_sqlite_pool() -> Result<DatabasePool, String> {
        let path = std::env::temp_dir().join(format!("scryr-test-{}.db", uuid::Uuid::new_v4()));
        connect_sqlite_path(&path).await.map_err(String::from)
    }

    #[test]
    fn modern_manifest_artifact_key_excludes_legacy_file_names() {
        assert!(is_modern_manifest_artifact_key("open_saas"));
        assert!(!is_modern_manifest_artifact_key(""));
        assert!(!is_modern_manifest_artifact_key("manifest.json"));
        assert!(!is_modern_manifest_artifact_key("plane/manifest.json"));
    }

    #[test]
    fn sqlite_url_path_resolution_matches_file_backed_sqlx_urls() {
        assert_eq!(
            sqlite_path_from_url("sqlite:data.db").as_deref(),
            Some(std::path::Path::new("data.db"))
        );
        assert_eq!(
            sqlite_path_from_url("sqlite://data.db?mode=rwc").as_deref(),
            Some(std::path::Path::new("data.db"))
        );
        assert_eq!(
            sqlite_path_from_url("sqlite:///tmp/scryr.db").as_deref(),
            Some(std::path::Path::new("/tmp/scryr.db"))
        );
        assert_eq!(
            sqlite_path_from_url("sqlite://space%20name.db").as_deref(),
            Some(std::path::Path::new("space name.db"))
        );
    }

    #[test]
    fn sqlite_url_path_resolution_skips_memory_databases() {
        assert_eq!(sqlite_path_from_url("sqlite::memory:"), None);
        assert_eq!(sqlite_path_from_url("sqlite://?mode=memory"), None);
        assert_eq!(
            sqlite_path_from_url("sqlite://data.db?mode=memory&cache=shared"),
            None
        );
    }

    #[tokio::test]
    async fn generated_manifest_value_artifacts_require_source_files_and_line_numbers()
    -> Result<(), String> {
        let pool = test_sqlite_pool().await?;

        let ensure_result = ensure_table(&pool).await;
        assert!(
            ensure_result.is_ok(),
            "generated_manifests table should be available: {ensure_result:?}"
        );

        let DatabasePool::Sqlite(sqlite_pool) = &pool else {
            return Err("expected SQLite test pool".to_string());
        };
        sqlx::query(
            r"
            INSERT INTO generated_manifests (
                id, artifact_kind, manifeset_file_name, clerk_org_id, content
            )
            VALUES (?, ?, ?, ?, ?)
            ",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(ArtifactKind::Value.as_str())
        .bind("sample")
        .bind("org")
        .bind(
            r#"{
                "files": [{"path":"index.scry","content":"sample = Manifest(name='Sample')"}],
                "manifests": [{"name":"Sample","line_number":1}]
            }"#,
        )
        .execute(sqlite_pool)
        .await
        .map_err(|error| format!("failed to insert SQLite sample row: {error}"))?;

        let rows_result = sqlx::query(
            r"
            SELECT manifeset_file_name, content
            FROM generated_manifests
            WHERE artifact_kind = ?
            ORDER BY manifeset_file_name ASC
            ",
        )
        .bind(ArtifactKind::Value.as_str())
        .fetch_all(sqlite_pool)
        .await;

        let rows =
            rows_result.map_err(|error| format!("value artifacts should be queryable: {error}"))?;

        assert!(!rows.is_empty(), "expected at least one value artifact row");

        for row in rows {
            let artifact_key_result = row.try_get::<String, _>("manifeset_file_name");
            assert!(
                artifact_key_result.is_ok(),
                "row should include manifest file name: {artifact_key_result:?}"
            );
            let artifact_key = artifact_key_result.unwrap_or_default();

            let content_result = row.try_get::<String, _>("content");
            assert!(
                content_result.is_ok(),
                "row should include content: {content_result:?}"
            );
            let content = content_result.unwrap_or_default();

            let validation = parse_manifest_blocks_value(&artifact_key, &content);
            if let Err(error) = validation {
                assert!(
                    error.is_empty(),
                    "artifact {artifact_key} failed validation: {error}"
                );
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn sample_lookup_falls_back_to_unowned_legacy_rows() -> Result<(), String> {
        let pool = test_sqlite_pool().await?;

        ensure_table(&pool).await?;
        let test_run_id = uuid::Uuid::new_v4();
        let sample_key = format!("sample-fallback-{test_run_id}");
        let content = r#"{
            "files": [{"path":"index.scry","content":"sample = Manifest(name='Sample')"}],
            "manifests": [{"name":"Sample","line_number":1}]
        }"#;

        let DatabasePool::Sqlite(sqlite_pool) = &pool else {
            return Err("expected SQLite test pool".to_string());
        };
        sqlx::query(
            r"
            INSERT INTO generated_manifests (
                id, artifact_kind, manifeset_file_name, clerk_org_id, content
            )
            VALUES (?, ?, ?, '', ?)
            ",
        )
        .bind(test_run_id.to_string())
        .bind(ArtifactKind::Value.as_str())
        .bind(&sample_key)
        .bind(content)
        .execute(sqlite_pool)
        .await
        .map_err(|error| format!("failed to insert unowned sample row: {error}"))?;

        let value =
            read_generated_manifest_json(&pool, "org_without_sample", Some(&sample_key)).await?;

        assert_eq!(
            value,
            serde_json::json!([{ "name": "Sample", "line_number": 1 }])
        );
        Ok(())
    }
}
