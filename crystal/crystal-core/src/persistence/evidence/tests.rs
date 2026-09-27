use super::*;
use crate::collectors::declarations;
use serde_json::json;

fn principal(org: &str) -> ManifestRequestContext {
    ManifestRequestContext {
        clerk_user_id: "user".into(),
        clerk_org_id: org.into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:admin".into()),
        clerk_org_permissions: vec![],
    }
}
fn config() -> Result<Vec<CollectorDeclaration>, String> {
    declarations(
        &json!([{"manifestId":"api","repository":[{"kind":"git_status","id":"git"},{"kind":"github_actions","id":"actions","repository":"example/api"}]}]),
    )
}
fn observation(id: &str, revision: &str, time: &str) -> Result<EvidenceObservation, String> {
    serde_json::from_value(json!({"schemaVersion":1,"observationId":id,"manifestId":"api","section":"repository","collectorId":"git","integration":"git_status","workspaceId":"laptop","scope":"worktree","planRevision":"plan","collectorRevision":revision,"runId":id,"attempt":1,"observedAt":time,"startedAt":time,"inputFingerprint":"input","result":{"kind":"git","data":{"branch":"main","commit":"abc","dirty":false,"changedFiles":0,"ahead":0,"behind":0}}})).map_err(|e|e.to_string())
}
fn status(revision: &str, time: &str, state: CollectorState) -> Result<CollectorStatus, String> {
    serde_json::from_value(json!({"manifestId":"api","section":"repository","collectorId":"git","workspaceId":"laptop","integration":"git_status","collectorRevision":revision,"state":state,"updatedAt":time,"freshnessSeconds":3600,"inputFingerprint":"input"})).map_err(|e|e.to_string())
}

fn workflow_run(
    run_id: &str,
    attempt: u32,
    status: &str,
    updated_at: chrono::DateTime<Utc>,
) -> crate::evidence::WorkflowRun {
    crate::evidence::WorkflowRun {
        run_id: run_id.into(),
        attempt,
        name: "CI".into(),
        status: status.into(),
        conclusion: (status == "completed").then(|| "success".into()),
        branch: "main".into(),
        commit: "abc".into(),
        url: "https://github.com/example/api/actions".into(),
        updated_at,
    }
}
fn workflows_observation(
    id: &str,
    revision: &str,
    observed_at: chrono::DateTime<Utc>,
    items: Vec<crate::evidence::WorkflowRun>,
) -> Result<EvidenceObservation, String> {
    let mut observation = observation(id, revision, &time(observed_at))?;
    observation.collector_id = "actions".into();
    observation.integration = "github_actions".into();
    observation.result =
        crate::evidence::EvidenceResult::Workflows(crate::evidence::WorkflowsResult {
            repository: "example/api".into(),
            items,
            complete: true,
        });
    Ok(observation)
}
fn workflow_items(
    observation: &EvidenceObservation,
) -> Result<&[crate::evidence::WorkflowRun], String> {
    match &observation.result {
        crate::evidence::EvidenceResult::Workflows(result) => Ok(&result.items),
        _ => Err("expected workflow result".into()),
    }
}

#[tokio::test]
async fn workflow_projection_rejects_late_provider_regressions_without_rewriting_history()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pool = super::super::connect_sqlite_path(&directory.path().join("workflows.db")).await?;
    let cfg = config()?;
    let now = Utc::now();
    let done = workflows_observation(
        "done",
        &cfg[1].revision,
        now,
        vec![workflow_run("run", 1, "completed", now)],
    )?;
    assert!(record_evidence(&pool, &principal("org"), done.clone()).await?);
    assert!(!record_evidence(&pool, &principal("org"), done).await?);
    let late = workflows_observation(
        "late",
        &cfg[1].revision,
        now + chrono::Duration::seconds(1),
        vec![workflow_run(
            "run",
            1,
            "in_progress",
            now - chrono::Duration::seconds(10),
        )],
    )?;
    record_evidence(&pool, &principal("org"), late.clone()).await?;
    let projection = project_evidence(&pool, "org", &cfg, Some("laptop")).await?;
    let current = projection[1]
        .latest
        .as_ref()
        .ok_or("missing workflow projection")?;
    assert_eq!(workflow_items(current)?[0].status, "completed");
    assert_eq!(current.observation_id, "late");
    let immutable = evidence_observation(&pool, "org", "late", Some("laptop"))
        .await?
        .ok_or("missing history")?;
    assert_eq!(workflow_items(&immutable)?[0].status, "in_progress");
    assert!(!record_evidence(&pool, &principal("org"), late.clone()).await?);

    // The same provider identifiers in another tenant have independent state.
    record_evidence(&pool, &principal("other"), late.clone()).await?;
    let other = project_evidence(&pool, "other", &cfg, Some("laptop")).await?;
    assert_eq!(
        workflow_items(other[1].latest.as_ref().ok_or("missing other tenant")?)?[0].status,
        "in_progress"
    );
    let mut collision = late;
    collision.result =
        crate::evidence::EvidenceResult::Workflows(crate::evidence::WorkflowsResult {
            repository: "example/api".into(),
            items: vec![workflow_run("run", 2, "completed", now)],
            complete: true,
        });
    assert!(
        record_evidence(&pool, &principal("org"), collision)
            .await
            .is_err()
    );
    assert_eq!(
        query(
            &pool,
            "SELECT CAST(COUNT(*) AS TEXT) FROM workflow_run_projection",
            vec![]
        )
        .await?,
        vec!["2"]
    );
    Ok(())
}

#[tokio::test]
async fn workflow_terminal_ties_win_and_rerun_attempts_remain_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pool =
        super::super::connect_sqlite_path(&directory.path().join("workflow-ties.db")).await?;
    let cfg = config()?;
    let now = Utc::now();
    for (index, status) in ["queued", "in_progress", "completed", "in_progress"]
        .iter()
        .enumerate()
    {
        let observation = workflows_observation(
            &format!("snapshot-{index}"),
            &cfg[1].revision,
            now + chrono::Duration::seconds(i64::try_from(index)?),
            vec![workflow_run("run", 1, status, now)],
        )?;
        record_evidence(&pool, &principal("org"), observation).await?;
    }
    let rerun = workflows_observation(
        "rerun",
        &cfg[1].revision,
        now + chrono::Duration::seconds(10),
        vec![
            workflow_run("run", 1, "queued", now),
            workflow_run("run", 2, "queued", now),
        ],
    )?;
    record_evidence(&pool, &principal("org"), rerun).await?;
    let projection = project_evidence(&pool, "org", &cfg, Some("laptop")).await?;
    let runs = workflow_items(
        projection[1]
            .latest
            .as_ref()
            .ok_or("missing workflow projection")?,
    )?;
    assert_eq!((runs[0].attempt, runs[0].status.as_str()), (1, "completed"));
    assert_eq!((runs[1].attempt, runs[1].status.as_str()), (2, "queued"));
    Ok(())
}

#[tokio::test]
async fn workflow_retention_removes_orphaned_sources_but_keeps_latest_terminal_state()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pool =
        super::super::connect_sqlite_path(&directory.path().join("workflow-retention.db")).await?;
    let cfg = config()?;
    let now = Utc::now();
    let old = now - chrono::Duration::days(31);
    let observation = workflows_observation(
        "old-source",
        "old-revision",
        old,
        vec![workflow_run("old-run", 1, "completed", old)],
    )?;
    record_evidence(&pool, &principal("org"), observation).await?;
    assert_eq!(
        query(
            &pool,
            "SELECT CAST(COUNT(*) AS TEXT) FROM workflow_run_projection",
            vec![]
        )
        .await?,
        vec!["1"]
    );
    let current = workflows_observation(
        "current",
        &cfg[1].revision,
        now,
        vec![workflow_run("current-run", 1, "completed", old)],
    )?;
    record_evidence(&pool, &principal("org"), current).await?;
    assert!(
        evidence_observation(&pool, "org", "old-source", None)
            .await?
            .is_none()
    );
    assert_eq!(
        query(
            &pool,
            "SELECT CAST(COUNT(*) AS TEXT) FROM workflow_run_projection",
            vec![]
        )
        .await?,
        vec!["1"]
    );
    let stale = workflows_observation(
        "late-current",
        &cfg[1].revision,
        now + chrono::Duration::seconds(1),
        vec![workflow_run("current-run", 1, "queued", old)],
    )?;
    record_evidence(&pool, &principal("org"), stale).await?;
    let projection = project_evidence(&pool, "org", &cfg, Some("laptop")).await?;
    assert_eq!(
        workflow_items(projection[1].latest.as_ref().ok_or("missing current")?)?[0].status,
        "completed"
    );
    Ok(())
}

#[tokio::test]
async fn immutable_delivery_is_durable_ordered_and_tenant_isolated()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("evidence.db");
    let pool = super::super::connect_sqlite_path(&path).await?;
    let cfg = config()?;
    let revision = &cfg[0].revision;
    let newer = observation("new", revision, &time(Utc::now()))?;
    assert!(record_evidence(&pool, &principal("org"), newer.clone()).await?);
    assert!(!record_evidence(&pool, &principal("org"), newer.clone()).await?);
    let mut collision = newer.clone();
    collision.input_fingerprint = "different".into();
    assert!(
        record_evidence(&pool, &principal("org"), collision)
            .await
            .is_err()
    );
    assert!(
        record_evidence(
            &pool,
            &principal("org"),
            observation(
                "old",
                revision,
                &time(Utc::now() - chrono::Duration::hours(1))
            )?
        )
        .await?
    );
    let mut denied = principal("org");
    denied.clerk_org_role = Some("org:viewer".into());
    assert!(record_evidence(&pool, &denied, newer).await.is_err());
    assert!(
        evidence_history(
            &pool,
            "other",
            "api",
            EvidenceSection::Repository,
            "git",
            "laptop",
            50,
            0
        )
        .await?
        .is_empty()
    );
    assert!(
        evidence_history(
            &pool,
            "org",
            "api",
            EvidenceSection::Repository,
            "git",
            "other-laptop",
            50,
            0
        )
        .await?
        .is_empty()
    );
    drop(pool);
    let pool = super::super::connect_sqlite_path(&path).await?;
    let history = evidence_history(
        &pool,
        "org",
        "api",
        EvidenceSection::Repository,
        "git",
        "laptop",
        50,
        0,
    )
    .await?;
    assert_eq!(
        history
            .iter()
            .map(|v| v.observation_id.as_str())
            .collect::<Vec<_>>(),
        vec!["new", "old"]
    );
    assert!(history[0].recorded_at.is_some());
    Ok(())
}
#[tokio::test]
async fn projections_keep_declaration_order_status_and_last_good_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pool = super::super::connect_sqlite_path(&directory.path().join("projection.db")).await?;
    let cfg = config()?;
    let waiting = project_evidence(&pool, "org", &cfg, Some("laptop")).await?;
    assert_eq!(
        waiting
            .iter()
            .map(|v| v.collector_id.as_str())
            .collect::<Vec<_>>(),
        vec!["git", "actions"]
    );
    assert!(
        waiting
            .iter()
            .all(|v| v.state == CollectorState::Waiting && v.latest.is_none())
    );
    let revision = &cfg[0].revision;
    record_evidence(
        &pool,
        &principal("org"),
        observation("result", revision, "2026-09-17T11:00:00Z")?,
    )
    .await?;
    assert!(
        record_collector_status(
            &pool,
            &principal("org"),
            status(revision, "2026-09-17T12:00:00Z", CollectorState::Error)?
        )
        .await?
    );
    assert!(
        !record_collector_status(
            &pool,
            &principal("org"),
            status(revision, "2026-09-17T10:00:00Z", CollectorState::Running)?
        )
        .await?
    );
    let evidence = project_evidence(&pool, "org", &cfg, Some("laptop")).await?;
    assert_eq!(evidence[0].state, CollectorState::Error);
    assert!(evidence[0].latest.is_some());
    assert!(!evidence[0].outdated);
    let mut cfg = cfg;
    cfg[0].revision = "new-revision".into();
    let changed = project_evidence(&pool, "org", &cfg, Some("laptop")).await?;
    assert!(changed[0].outdated);
    assert_eq!(changed[0].state, CollectorState::Waiting);
    let isolated = project_evidence(&pool, "org", &cfg, Some("desktop")).await?;
    assert!(isolated.iter().all(|v| v.latest.is_none()));
    Ok(())
}
#[tokio::test]
async fn old_artifact_collected_now_remains_stale() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pool = super::super::connect_sqlite_path(&directory.path().join("artifact.db")).await?;
    let cfg = declarations(
        &json!([{"manifestId":"api","tests":[{"kind":"lcov","id":"coverage","files":["coverage.lcov"],"freshness":3600}]}]),
    )?;
    let now = Utc::now();
    let mut report = observation("imported", &cfg[0].revision, &time(now))?;
    assert_eq!(report.source_updated_at, None);
    report.collector_id = "coverage".into();
    report.section = EvidenceSection::Tests;
    report.integration = "lcov".into();
    report.result = crate::evidence::EvidenceResult::Coverage(crate::evidence::CoverageResult {
        suite: "unit".into(),
        covered: 1,
        total: 2,
    });
    report.source_updated_at = Some(now - chrono::Duration::hours(24));
    record_evidence(&pool, &principal("org"), report.clone()).await?;
    let evidence = project_evidence(&pool, "org", &cfg, Some("laptop")).await?;
    assert!(evidence[0].stale);
    assert!(!evidence[0].outdated);
    let latest = evidence[0]
        .latest
        .as_ref()
        .ok_or("missing imported evidence")?;
    assert_eq!(latest.source_updated_at, report.source_updated_at);
    assert_eq!(latest.observed_at, report.observed_at);
    assert!(serde_json::to_value(latest)?["sourceUpdatedAt"].is_string());

    report.observation_id = "fresh".into();
    report.run_id = "fresh".into();
    report.observed_at += chrono::Duration::seconds(1);
    report.source_updated_at = None;
    record_evidence(&pool, &principal("org"), report).await?;
    assert!(!project_evidence(&pool, "org", &cfg, Some("laptop")).await?[0].stale);
    Ok(())
}
#[tokio::test]
async fn fresh_schema_initializes_atomically_and_legacy_data_is_untouched()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite.clone());
    let (a, b) = tokio::join!(schema::ensure_table(&pool), schema::ensure_table(&pool));
    a?;
    b?;
    assert_eq!(
        query(&pool, "SELECT version FROM scryr_schema", vec![]).await?,
        vec!["3"]
    );
    execute(&pool, "UPDATE scryr_schema SET version='2'", vec![]).await?;
    assert!(
        schema::ensure_table(&pool)
            .await
            .err()
            .is_some_and(|s| s.contains("Incompatible"))
    );
    assert_eq!(
        query(&pool, "SELECT version FROM scryr_schema", vec![]).await?,
        vec!["2"]
    );
    let legacy = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let legacy_pool = DatabasePool::Sqlite(legacy.clone());
    sqlx::query("CREATE TABLE generated_manifests (content TEXT)")
        .execute(&legacy)
        .await?;
    sqlx::query("INSERT INTO generated_manifests VALUES ('keep-me')")
        .execute(&legacy)
        .await?;
    assert!(
        schema::ensure_table(&legacy_pool)
            .await
            .err()
            .is_some_and(|s| s.contains("Existing data was not changed"))
    );
    assert_eq!(
        query(
            &legacy_pool,
            "SELECT content FROM generated_manifests",
            vec![]
        )
        .await?,
        vec!["keep-me"]
    );
    assert_eq!(
        query(
            &legacy_pool,
            "SELECT name FROM sqlite_master WHERE type='table'",
            vec![]
        )
        .await?,
        vec!["generated_manifests"]
    );
    Ok(())
}

#[tokio::test]
async fn dependency_projection_tracks_inventory_and_policy_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::evidence::EvidenceResult;
    let directory = tempfile::tempdir()?;
    let pool = super::super::connect_sqlite_path(&directory.path().join("upstream.db")).await?;
    let cfg = declarations(
        &json!([{"manifestId":"inventory","dependencies":[{"kind":"syft_inventory","id":"sbom"}]},{"manifestId":"api","dependencies":[{"kind":"grype_scan","id":"vulns","sbom":{"manifest_id":"inventory","collector_id":"sbom"}},{"kind":"grant_license","id":"licenses","sbom":{"manifest_id":"inventory","collector_id":"sbom"}}]}]),
    )?;
    let mut inventory = observation("inv1", &cfg[0].revision, &time(Utc::now()))?;
    inventory.manifest_id = "inventory".into();
    inventory.section = EvidenceSection::Dependencies;
    inventory.collector_id = "sbom".into();
    inventory.integration = "syft_inventory".into();
    inventory.result = serde_json::from_value(
        json!({"kind":"inventory","data":{"packages":[],"relationships":[],"complete":true,"artifactHash":"hash1"}}),
    )?;
    let mut scan = observation("scan", &cfg[1].revision, &time(Utc::now()))?;
    scan.section = EvidenceSection::Dependencies;
    scan.collector_id = "vulns".into();
    scan.integration = "grype_scan".into();
    scan.upstream_fingerprint = Some("hash1".into());
    scan.result = serde_json::from_value(
        json!({"kind":"vulnerability","data":{"items":[],"inventoryHash":"hash1","complete":true}}),
    )?;
    let crate::collectors::CollectorConfig::GrantLicense(config) = &cfg[2].config else {
        return Err("expected policy".into());
    };
    let policy_revision = crate::collectors::fingerprint(&config.policy)?;
    let mut license = scan.clone();
    license.observation_id = "license".into();
    license.run_id = "license".into();
    license.collector_id = "licenses".into();
    license.integration = "grant_license".into();
    license.collector_revision = cfg[2].revision.clone();
    license.policy_revision = Some(policy_revision.clone());
    license.result = serde_json::from_value(
        json!({"kind":"license","data":{"items":[],"inventoryHash":"hash1","policyRevision":policy_revision,"complete":true}}),
    )?;
    record_evidence(&pool, &principal("org"), inventory.clone()).await?;
    record_evidence(&pool, &principal("org"), scan).await?;
    record_evidence(&pool, &principal("org"), license).await?;
    // The selected diagram need not include the referenced inventory manifest.
    let projected = project_evidence(&pool, "org", &cfg[1..], Some("laptop")).await?;
    assert!(projected.iter().all(|e| !e.outdated));
    inventory.observation_id = "inv2".into();
    inventory.run_id = "inv2".into();
    inventory.observed_at += chrono::Duration::seconds(1);
    if let EvidenceResult::Inventory(result) = &mut inventory.result {
        result.artifact_hash = "hash2".into();
    }
    record_evidence(&pool, &principal("org"), inventory).await?;
    let projected = project_evidence(&pool, "org", &cfg[1..], Some("laptop")).await?;
    assert!(projected.iter().all(|e| e.outdated));
    Ok(())
}
#[tokio::test]
async fn retention_preserves_last_success_and_expires_older_history()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pool = super::super::connect_sqlite_path(&directory.path().join("retention.db")).await?;
    let cfg = config()?;
    let old = Utc::now() - chrono::Duration::days(60);
    record_evidence(
        &pool,
        &principal("org"),
        observation("old", &cfg[0].revision, &time(old))?,
    )
    .await?;
    record_evidence(
        &pool,
        &principal("org"),
        observation(
            "newer",
            &cfg[0].revision,
            &time(old + chrono::Duration::hours(1)),
        )?,
    )
    .await?;
    let history = evidence_history(
        &pool,
        "org",
        "api",
        EvidenceSection::Repository,
        "git",
        "laptop",
        50,
        0,
    )
    .await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].observation_id, "newer");
    Ok(())
}
