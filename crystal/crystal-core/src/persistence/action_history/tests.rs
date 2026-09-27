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

#[tokio::test]
async fn job_snapshot_enriches_a_run_and_retries_are_idempotent()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite);
    let context = principal("org");
    let mut completed = run("completed", "2026-09-08T10:02:00Z", 1)?;
    assert!(
        record_action_run(
            &pool,
            &context,
            "services/api",
            completed.clone(),
            None,
            "webhook"
        )
        .await?
    );
    completed.jobs = Some(serde_json::from_value(serde_json::json!([{
        "id": 99, "name":"tests", "status":"completed", "conclusion":"success",
        "startedAt":"2026-09-08T10:00:00Z", "completedAt":"2026-09-08T10:02:00Z",
        "htmlUrl":"https://github.com/example/api/actions/runs/12345/jobs/99"
    }]))?);
    assert!(
        record_action_run(
            &pool,
            &context,
            "services/api",
            completed.clone(),
            None,
            "webhook"
        )
        .await?
    );
    assert!(!record_action_run(&pool, &context, "services/api", completed, None, "webhook").await?);
    let history = read_action_history(&pool, "org", "services/api", 100, 0).await?;
    assert_eq!(history.runs.len(), 1);
    assert_eq!(history.runs[0].jobs.as_ref().map(Vec::len), Some(1));
    Ok(())
}

#[tokio::test]
async fn api_polls_enrich_jobs_and_workflow_path_without_new_transitions()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite.clone());
    let context = principal("org");
    let mut active = run("in_progress", "2026-09-08T10:01:00Z", 1)?;
    active.jobs = Some(serde_json::from_value(serde_json::json!([{
        "id":99, "name":"tests", "status":"queued", "conclusion":null,
        "startedAt":null, "completedAt":null,
        "htmlUrl":"https://github.com/example/api/actions/runs/12345/jobs/99"
    }]))?);
    assert!(record_action_run(&pool, &context, "api", active.clone(), None, "api").await?);
    let stale = active.clone();
    active.updated_at = "2026-09-08T10:02:00Z".parse()?;
    if let Some(jobs) = &mut active.jobs {
        jobs[0].status = "in_progress".into();
    }
    assert!(record_action_run(&pool, &context, "api", active.clone(), None, "api").await?);
    assert!(!record_action_run(&pool, &context, "api", active.clone(), None, "api").await?);
    assert!(!record_action_run(&pool, &context, "api", stale, None, "api").await?);
    let current = read_action_history(&pool, "org", "api", 100, 0).await?;
    assert_eq!(current.runs[0].events.len(), 1);
    assert_eq!(current.runs[0].updated_at, active.updated_at);
    assert_eq!(
        current.runs[0]
            .jobs
            .as_ref()
            .map(|jobs| jobs[0].status.as_str()),
        Some("in_progress")
    );
    active.workflow_path = Some(".github/workflows/ci.yml".into());
    assert!(record_action_run(&pool, &context, "api", active.clone(), None, "api").await?);
    assert!(!record_action_run(&pool, &context, "api", active, None, "api").await?);
    let enriched = read_action_history(&pool, "org", "api", 100, 0).await?;
    assert_eq!(
        enriched.runs[0].workflow_path.as_deref(),
        Some(".github/workflows/ci.yml")
    );
    assert_eq!(enriched.runs[0].events.len(), 1);
    assert!(
        record_action_run(
            &pool,
            &context,
            "api",
            run("completed", "2026-09-08T10:03:00Z", 1)?,
            None,
            "api"
        )
        .await?
    );
    let complete = read_action_history(&pool, "org", "api", 100, 0).await?;
    assert_eq!(complete.runs[0].events.len(), 2);
    assert_eq!(
        complete.runs[0].workflow_path.as_deref(),
        Some(".github/workflows/ci.yml")
    );
    assert!(complete.runs[0].jobs.is_some());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM manifest_action_events")
            .fetch_one(&sqlite)
            .await?,
        2
    );
    Ok(())
}

#[test]
fn summaries_preserve_each_workflows_latest_outcome() -> Result<(), Box<dyn std::error::Error>> {
    let mut failing = run("completed", "2026-09-08T10:01:00Z", 1)?;
    failing.conclusion = Some("failure".into());
    assert!(failing.workflow_path.is_none());
    assert!(
        serde_json::to_value(&failing)?
            .get("workflowPath")
            .is_none()
    );
    let mut passing = failing.clone();
    passing.workflow_id = 99;
    passing.run_id += 1;
    passing.conclusion = Some("success".into());
    let mut log = GithubActionsLog::default();
    log.merge(failing.clone());
    log.merge(passing.clone());
    assert_eq!(log.build_status(), Some("failing"));
    failing.run_attempt = 2;
    failing.status = "in_progress".into();
    failing.conclusion = None;
    log.merge(failing.clone());
    assert_eq!(log.build_status(), Some("pending"));
    failing.status = "completed".into();
    failing.conclusion = Some("cancelled".into());
    log.merge(failing.clone());
    assert_eq!(log.build_status(), None);
    failing.run_attempt = 3;
    failing.conclusion = Some("success".into());
    log.merge(failing.clone());
    assert_eq!(log.build_status(), Some("passing"));
    failing.head_branch = Some("other".into());
    failing.run_id += 2;
    failing.conclusion = Some("failure".into());
    log.merge(failing);
    assert_eq!(log.build_status(), Some("failing"));
    passing.host = "enterprise.example".into();
    passing.conclusion = Some("failure".into());
    log.merge(passing);
    assert_eq!(log.build_status(), Some("failing"));
    Ok(())
}

#[tokio::test]
async fn api_snapshot_times_compare_fractional_seconds_chronologically()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite);
    let context = principal("org");
    // Old records use variable-width UTC fractions. Exercise every supported
    // width plus nanosecond changes that SQLite datetime functions would round.
    for (index, (before, after)) in [
        ("2026-09-08T10:01:00Z", "2026-09-08T10:01:00.500Z"),
        ("2026-09-08T10:01:00.500Z", "2026-09-08T10:01:00.500001Z"),
        (
            "2026-09-08T10:01:00.500001Z",
            "2026-09-08T10:01:00.500001001Z",
        ),
        (
            "2026-09-08T10:01:00.500001001Z",
            "2026-09-08T10:01:00.500001002Z",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let manifest = format!("api/{index}");
        let mut observation = run("in_progress", before, 1)?;
        observation.jobs = Some(Vec::new());
        assert!(
            record_action_run(&pool, &context, &manifest, observation.clone(), None, "api").await?
        );
        let stale = observation.clone();
        observation.updated_at = after.parse()?;
        observation.jobs = Some(serde_json::from_value(serde_json::json!([{
            "id":99, "name":"tests", "status":"in_progress", "conclusion":null,
            "startedAt":null, "completedAt":null,
            "htmlUrl":"https://github.com/example/api/actions/runs/12345/jobs/99"
        }]))?);
        assert!(
            record_action_run(&pool, &context, &manifest, observation.clone(), None, "api").await?
        );
        assert!(!record_action_run(&pool, &context, &manifest, stale.clone(), None, "api").await?);
        // An older path-only enrichment must preserve the newer time and jobs.
        let mut older_metadata = stale;
        older_metadata.workflow_path = Some(".github/workflows/ci.yml".into());
        assert!(record_action_run(&pool, &context, &manifest, older_metadata, None, "api").await?);
        let history = read_action_history(&pool, "org", &manifest, 100, 0).await?;
        assert_eq!(history.runs[0].updated_at, observation.updated_at);
        assert_eq!(history.runs[0].jobs.as_ref().map(Vec::len), Some(1));
        assert_eq!(
            history.runs[0].workflow_path.as_deref(),
            Some(".github/workflows/ci.yml")
        );
        assert_eq!(history.runs[0].events.len(), 1);
    }
    Ok(())
}
