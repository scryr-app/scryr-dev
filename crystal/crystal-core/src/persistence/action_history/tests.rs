//! Persistence behavior shared by the GraphQL and SDK reporting paths.

use super::*;
use crate::manifest::{ArtifactKind, UpsertGeneratedManifestInput};
use crate::persistence::{
    persist_generated_manifest, read_generated_manifest_json_by_scry_identifier,
};

fn principal(org: &str) -> ManifestRequestContext {
    ManifestRequestContext {
        clerk_user_id: "user".into(),
        clerk_org_id: org.into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:admin".into()),
        clerk_org_permissions: vec![],
    }
}

fn run(status: &str, updated: &str, attempt: u64) -> Result<GithubActionRun, serde_json::Error> {
    serde_json::from_value(serde_json::json!({
        "host": "github.com", "repositoryId": 123, "repository": "example/api",
        "workflowId": 42, "workflowName": "CI", "runId": 12345, "runAttempt": attempt,
        "headBranch": "main", "headSha": "abcdef", "htmlUrl": "https://github.com/example/api/actions/runs/12345",
        "status": status, "conclusion": if status == "completed" { Some("success") } else { None },
        "createdAt": "2026-09-08T10:00:00Z", "updatedAt": updated
    }))
}

#[tokio::test]
#[allow(clippy::too_many_lines)] // One lifecycle: delivery, rerun, regeneration, and isolation.
async fn history_survives_regeneration_and_is_tenant_scoped()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite.clone());
    let context = principal("org");
    let completed = run("completed", "2026-09-08T10:02:00Z", 1)?;
    assert!(
        record_action_run(
            &pool,
            &context,
            "services/api",
            completed.clone(),
            Some("completed".into()),
            "webhook"
        )
        .await?
    );
    assert!(
        !record_action_run(
            &pool,
            &context,
            "services/api",
            completed.clone(),
            Some("completed".into()),
            "webhook"
        )
        .await?
    );
    // Polling deduplicates the same state even when GitHub's updated_at changes.
    assert!(
        !record_action_run(
            &pool,
            &context,
            "services/api",
            run("completed", "2026-09-08T10:03:00Z", 1)?,
            None,
            "api"
        )
        .await?
    );
    assert!(
        record_action_run(
            &pool,
            &context,
            "services/api",
            run("queued", "2026-09-08T10:00:00Z", 1)?,
            None,
            "api"
        )
        .await?
    );
    assert!(
        record_action_run(
            &pool,
            &context,
            "services/api",
            run("in_progress", "2026-09-08T10:01:00Z", 1)?,
            None,
            "api"
        )
        .await?
    );
    assert!(
        record_action_run(
            &pool,
            &context,
            "services/api",
            run("queued", "2026-09-08T11:00:00Z", 2)?,
            None,
            "api"
        )
        .await?
    );
    let log = read_action_history(&pool, "org", "services/api", 100, 0).await?;
    assert_eq!(log.runs.len(), 2);
    assert_eq!(log.runs[0].run_attempt, 2);
    assert_eq!(log.runs[1].status, "completed");
    assert_eq!(log.runs[1].events.len(), 3);
    assert_eq!(log.runs[1].events[0].status, "queued");
    assert!(
        read_action_history(&pool, "other-org", "services/api", 100, 0)
            .await?
            .runs
            .is_empty()
    );
    assert_eq!(
        read_action_history(&pool, "org", "services/api", 1, 1)
            .await?
            .runs[0]
            .run_attempt,
        1
    );
    let mut input = UpsertGeneratedManifestInput {
        artifact_kind: ArtifactKind::Value, artifact_key: "api".into(), folder_path: None,
        file_name: None, scry_identifier: Some("api".into()), name: Some("API".into()),
        git_commit_sha: None, content: serde_json::json!({
            "files": [{"path": "index.scry", "content": "api = Manifest(manifest_id='services/api')"}],
            "manifests": [{"name": "API", "manifestId": "services/api", "line_number": 1}]
        }).to_string(),
    };
    persist_generated_manifest(&pool, &input, &context).await?;
    let first = read_generated_manifest_json_by_scry_identifier(&pool, "org", "api").await?;
    assert!(first[0].get("cicd").is_none_or(serde_json::Value::is_null));
    input.content = input.content.replace("\"API\"", "\"Renamed API\"");
    persist_generated_manifest(&pool, &input, &context).await?;
    let updated = read_generated_manifest_json_by_scry_identifier(&pool, "org", "api").await?;
    assert_eq!(updated[0]["name"], "Renamed API");
    assert!(
        updated[0]
            .get("cicd")
            .is_none_or(serde_json::Value::is_null)
    );
    assert_eq!(first[0]["cicd"], updated[0]["cicd"]);
    let mut denied = principal("org");
    denied.clerk_org_role = None;
    assert!(
        record_action_run(&pool, &denied, "services/api", completed, None, "api")
            .await
            .is_err()
    );
    sqlite.close().await;
    Ok(())
}

#[tokio::test]
async fn concurrent_deliveries_append_without_lost_updates()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite.clone());
    schema::ensure_table(&pool).await?;
    let context = principal("org");
    let queued = run("queued", "2026-09-08T10:00:00Z", 1)?;
    let completed = run("completed", "2026-09-08T10:02:00Z", 1)?;
    let (first, second, duplicate) = tokio::join!(
        record_action_run(&pool, &context, "services/api", queued, None, "api"),
        record_action_run(
            &pool,
            &context,
            "services/api",
            completed.clone(),
            None,
            "api"
        ),
        record_action_run(&pool, &context, "services/api", completed, None, "api"),
    );
    assert!(first?);
    assert_ne!(second?, duplicate?);
    let log = read_action_history(&pool, "org", "services/api", 100, 0).await?;
    assert_eq!(log.runs[0].events.len(), 2);
    assert_eq!(log.runs[0].status, "completed");
    sqlite.close().await;
    Ok(())
}
