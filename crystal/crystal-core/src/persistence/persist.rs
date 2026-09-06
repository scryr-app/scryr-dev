//! Persistence orchestration for generated manifest writes.

use super::metadata::ManifestUploadMetadata;
use super::{sqlite_write, turso_write};
use crate::manifest::{ManifestRequestContext, UpsertGeneratedManifestInput};
use crate::persistence::{self, ArtifactKind, DatabasePool};
use uuid::Uuid;

/// Persist a generated manifest artifact using the shared SQL layer.
///
/// # Errors
///
/// Returns an error if schema bootstrap, upsert execution, or UUID parsing fails.
pub(crate) async fn persist_generated_manifest(
    pool: &DatabasePool,
    input: &UpsertGeneratedManifestInput,
    request_context: &ManifestRequestContext,
) -> Result<Uuid, String> {
    persistence::ensure_table(pool).await?;
    let metadata = ManifestUploadMetadata::from_input(input);
    let id = upsert_generated_manifest_row(pool, input, &metadata, request_context).await?;

    if input.artifact_kind == ArtifactKind::Value {
        persistence::cleanup_legacy_value_artifacts(pool, &request_context.clerk_org_id).await?;
    }

    Uuid::parse_str(&id).map_err(|error| {
        format!(
            "Failed to parse persisted generated artifact UUID for {}: {error}",
            input.artifact_key
        )
    })
}

/// Insert or update the current manifest row and append an upload ledger row.
async fn upsert_generated_manifest_row(
    pool: &DatabasePool,
    input: &UpsertGeneratedManifestInput,
    metadata: &ManifestUploadMetadata,
    request_context: &ManifestRequestContext,
) -> Result<String, String> {
    match pool {
        DatabasePool::Turso(database) => {
            turso_write::upsert_generated_manifest_row(database, input, metadata, request_context)
                .await
        }
        DatabasePool::Sqlite(pool) => {
            sqlite_write::upsert_generated_manifest_row(pool, input, metadata, request_context)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::persist_generated_manifest;
    use crate::manifest::{ManifestRequestContext, UpsertGeneratedManifestInput};
    use crate::persistence::{ArtifactKind, DatabasePool, connect_sqlite_path};
    use sqlx::Row;
    use uuid::Uuid;

    #[tokio::test]
    async fn persist_generated_manifest_updates_current_row_and_appends_ledger()
    -> Result<(), String> {
        let path = std::env::temp_dir().join(format!("scryr-test-{}.db", Uuid::new_v4()));
        let pool = connect_sqlite_path(&path).await?;
        let DatabasePool::Sqlite(sqlite_pool) = &pool else {
            return Err("expected SQLite test pool".to_string());
        };
        let test_run_id = Uuid::new_v4();
        let artifact_key = format!("test/{test_run_id}");
        let folder_path = format!("services/catalog/{test_run_id}");
        let request_context = ManifestRequestContext {
            clerk_user_id: "user_test".to_string(),
            clerk_org_id: format!("org_{test_run_id}"),
            clerk_org_slug: Some("test-org".to_string()),
            clerk_org_role: Some("org:admin".to_string()),
            clerk_org_permissions: Vec::new(),
        };
        let input = UpsertGeneratedManifestInput {
            artifact_kind: ArtifactKind::Schema,
            artifact_key,
            folder_path: Some(folder_path.clone()),
            file_name: Some("index.scry".to_string()),
            scry_identifier: Some(format!("catalog_diagram_{test_run_id}")),
            name: Some("Catalog".to_string()),
            git_commit_sha: Some("commit-one".to_string()),
            content: "{}".to_string(),
        };

        let current_id = persist_generated_manifest(&pool, &input, &request_context)
            .await
            .map_err(|error| {
                format!("first persist should write current and ledger rows: {error}")
            })?;
        let mut second_input = input.clone();
        second_input.git_commit_sha = Some("commit-two".to_string());
        second_input.content = r#"{"updated":true}"#.to_string();

        let second_current_id = persist_generated_manifest(&pool, &second_input, &request_context)
            .await
            .map_err(|error| {
                format!("second persist should update current and append ledger row: {error}")
            })?;

        assert_eq!(current_id, second_current_id);

        let current_row = sqlx::query(
            r"
            SELECT clerk_org_id, org_slug, uploaded_by_clerk_user_id, folder_path,
                   file_name, scry_identifier, name, git_commit_sha, content
            FROM generated_manifests
            WHERE id = ?
            ",
        )
        .bind(current_id.to_string())
        .fetch_one(sqlite_pool)
        .await
        .map_err(|error| format!("current row should be queryable: {error}"))?;

        assert_eq!(
            current_row.get::<String, _>("clerk_org_id"),
            request_context.clerk_org_id
        );
        assert_eq!(
            current_row.get::<Option<String>, _>("org_slug").as_deref(),
            Some("test-org")
        );
        assert_eq!(
            current_row.get::<String, _>("uploaded_by_clerk_user_id"),
            "user_test"
        );
        assert_eq!(current_row.get::<String, _>("folder_path"), folder_path);
        assert_eq!(current_row.get::<String, _>("file_name"), "index.scry");
        assert!(
            current_row
                .get::<String, _>("scry_identifier")
                .starts_with("catalog_diagram_")
        );
        assert_eq!(current_row.get::<String, _>("name"), "Catalog");
        assert_eq!(
            current_row
                .get::<Option<String>, _>("git_commit_sha")
                .as_deref(),
            Some("commit-two")
        );
        assert_eq!(
            current_row.get::<String, _>("content"),
            r#"{"updated":true}"#
        );

        let ledger_count = sqlx::query_scalar::<_, i64>(
            r"
            SELECT COUNT(*)
            FROM generated_manifest_uploads
            WHERE generated_manifest_id = ?
            ",
        )
        .bind(current_id.to_string())
        .fetch_one(sqlite_pool)
        .await
        .map_err(|error| format!("ledger rows should be queryable: {error}"))?;

        assert_eq!(ledger_count, 2);
        Ok(())
    }
}
