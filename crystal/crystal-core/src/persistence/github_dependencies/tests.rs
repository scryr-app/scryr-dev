//! Dependency component freshness, isolation, and validation regressions.

use super::*;
use crate::github_dependencies::{
    DependencyInventory, DependencyPackage, DependencySecurity, GithubDependencyAlert,
};

fn context(org: &str) -> ManifestRequestContext {
    ManifestRequestContext {
        clerk_user_id: "user".into(),
        clerk_org_id: org.into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:admin".into()),
        clerk_org_permissions: vec![],
    }
}

fn snapshot() -> Result<GithubDependencySnapshot, serde_json::Error> {
    serde_json::from_value(serde_json::json!({
        "repository":"example/api",
        "inventory":{"observedAt":"2026-09-08T10:00:00Z", "packages":[{"id":"pkg1","name":"react","version":"18","license":"MIT","purl":"pkg:npm/react@18"}],"directDeps":1},
        "security":{"observedAt":"2026-09-08T10:00:00Z", "alerts":[{"number":1,"package":"react","ecosystem":"npm","manifestPath":"package-lock.json","severity":"high","url":"https://github.com/example/api/security/dependabot/1","ghsaId":"GHSA-example","summary":"Example"}]}
    }))
}

async fn read(pool: &DatabasePool) -> Result<GithubDependencySnapshot, String> {
    read_github_dependencies(pool, "org", "api")
        .await?
        .ok_or_else(|| "missing snapshot".into())
}

#[tokio::test]
async fn partial_snapshots_are_independently_fresh_and_retries_are_idempotent()
-> Result<(), Box<dyn std::error::Error>> {
    let pool = DatabasePool::Sqlite(
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?,
    );
    let original = snapshot()?;
    assert!(record_github_dependencies(&pool, &context("org"), "api", original.clone()).await?);
    assert!(!record_github_dependencies(&pool, &context("org"), "api", original.clone()).await?);
    let mut inventory_only = original.clone();
    inventory_only.security = None;
    if let Some(inventory) = &mut inventory_only.inventory {
        inventory.observed_at = "2026-09-08T10:00:00.000000001Z".parse()?;
        inventory.packages[0].version = Some("19".into());
    }
    assert!(
        record_github_dependencies(&pool, &context("org"), "api", inventory_only.clone()).await?
    );
    let current = read(&pool).await?;
    assert_eq!(current.inventory, inventory_only.inventory);
    assert_eq!(current.security, original.security);
    assert!(!record_github_dependencies(&pool, &context("org"), "api", original.clone()).await?);
    let mut security_only = original.clone();
    security_only.inventory = None;
    if let Some(security) = &mut security_only.security {
        security.observed_at = "2026-09-08T10:01:00Z".parse()?;
        security.alerts.clear();
    }
    assert!(
        record_github_dependencies(&pool, &context("org"), "api", security_only.clone()).await?
    );
    let current = read(&pool).await?;
    assert_eq!(current.inventory, inventory_only.inventory);
    assert_eq!(current.security, security_only.security);
    // A mixed update can advance one component while its stale other component is ignored.
    let mut mixed = original;
    if let Some(inventory) = &mut mixed.inventory {
        inventory.observed_at = "2026-09-08T10:02:00Z".parse()?;
        inventory.packages.clear();
    }
    assert!(record_github_dependencies(&pool, &context("org"), "api", mixed.clone()).await?);
    let current = read(&pool).await?;
    assert_eq!(current.inventory, mixed.inventory);
    assert_eq!(current.security, security_only.security);
    Ok(())
}

#[tokio::test]
async fn repository_switch_clears_other_components_and_storage_survives_restart()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("dependencies.db");
    let pool = crate::persistence::connect_sqlite_path(&path).await?;
    record_github_dependencies(&pool, &context("org"), "api", snapshot()?).await?;
    let mut changed = snapshot()?;
    changed.repository = "Other/Project".into();
    changed.security = None;
    if let Some(inventory) = &mut changed.inventory {
        inventory.observed_at = "2026-09-08T10:01:00Z".parse()?;
    }
    assert!(record_github_dependencies(&pool, &context("org"), "api", changed.clone()).await?);
    changed.repository.make_ascii_lowercase();
    assert_eq!(read(&pool).await?, changed);
    // Late data from the old repository cannot roll back the new selection.
    assert!(!record_github_dependencies(&pool, &context("org"), "api", snapshot()?).await?);
    assert_eq!(read(&pool).await?, changed);
    if let DatabasePool::Sqlite(sqlite) = pool {
        sqlite.close().await;
    }
    let reopened = crate::persistence::connect_sqlite_path(&path).await?;
    assert_eq!(read(&reopened).await?, changed);
    assert!(
        read_github_dependencies(&reopened, "other", "api")
            .await?
            .is_none()
    );
    assert!(
        read_github_dependencies(&reopened, "org", "other")
            .await?
            .is_none()
    );
    assert!(
        read_github_dependencies(&reopened, "", "api")
            .await
            .is_err()
    );
    let mut denied = context("org");
    denied.clerk_org_role = None;
    assert!(
        record_github_dependencies(&reopened, &denied, "api", snapshot()?)
            .await
            .is_err()
    );
    assert!(
        record_github_dependencies(&reopened, &context(""), "api", snapshot()?)
            .await
            .is_err()
    );
    assert!(
        record_github_dependencies(&reopened, &context("org"), "", snapshot()?)
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn concurrent_partial_writes_preserve_both_components()
-> Result<(), Box<dyn std::error::Error>> {
    let pool = DatabasePool::Sqlite(
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?,
    );
    schema::ensure_table(&pool).await?;
    let original = snapshot()?;
    let mut inventory = original.clone();
    inventory.security = None;
    let mut security = original.clone();
    security.inventory = None;
    let principal = context("org");
    let (first, second) = tokio::join!(
        record_github_dependencies(&pool, &principal, "api", inventory),
        record_github_dependencies(&pool, &principal, "api", security)
    );
    assert!(first?);
    assert!(second?);
    assert_eq!(read(&pool).await?, original);
    Ok(())
}

#[test]
fn invalid_identities_urls_duplicates_and_bounds_are_rejected()
-> Result<(), Box<dyn std::error::Error>> {
    let original = serde_json::to_value(snapshot()?)?;
    for repository in [
        "../api",
        "owner/..",
        "https://github.com/owner/repo",
        "owner/repo?token=x",
        "owner/repo/extra",
        "owner /repo",
    ] {
        let mut invalid = original.clone();
        invalid["repository"] = serde_json::json!(repository);
        assert!(
            serde_json::from_value::<GithubDependencySnapshot>(invalid)?
                .validate()
                .is_err()
        );
    }
    for url in [
        "http://github.com/example/api/security/dependabot/1",
        "https://evil.example/example/api/security/dependabot/1",
        "https://github.com/example/api/security/dependabot/1?token=x",
        "https://github.com/example/other/security/dependabot/1",
        "https://github.com/example/api/security/dependabot/2",
    ] {
        let mut invalid = original.clone();
        invalid["security"]["alerts"][0]["url"] = serde_json::json!(url);
        assert!(
            serde_json::from_value::<GithubDependencySnapshot>(invalid)?
                .validate()
                .is_err()
        );
    }
    for (field, value) in [
        ("number", serde_json::json!(0)),
        ("severity", serde_json::json!("moderate")),
        ("package", serde_json::json!("")),
        ("manifestPath", serde_json::json!("../package-lock.json")),
    ] {
        let mut invalid = original.clone();
        invalid["security"]["alerts"][0][field] = value;
        assert!(
            serde_json::from_value::<GithubDependencySnapshot>(invalid)?
                .validate()
                .is_err()
        );
    }
    let mut invalid = original.clone();
    invalid["inventory"]["packages"] = serde_json::json!([
        original["inventory"]["packages"][0],
        original["inventory"]["packages"][0]
    ]);
    assert!(
        serde_json::from_value::<GithubDependencySnapshot>(invalid)?
            .validate()
            .is_err()
    );
    let mut invalid = original.clone();
    invalid["security"]["alerts"] = serde_json::json!([
        original["security"]["alerts"][0],
        original["security"]["alerts"][0]
    ]);
    assert!(
        serde_json::from_value::<GithubDependencySnapshot>(invalid)?
            .validate()
            .is_err()
    );
    let mut empty = snapshot()?;
    empty.inventory = None;
    empty.security = None;
    assert!(empty.validate().is_err());
    let mut oversized = original;
    oversized["security"]["alerts"][0]["summary"] = serde_json::json!("x".repeat(2_000_000));
    assert!(
        serde_json::from_value::<GithubDependencySnapshot>(oversized)?
            .validate()
            .is_err()
    );
    Ok(())
}

#[test]
fn snapshot_accepts_empty_complete_components_and_enforces_collection_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();
    let mut value = GithubDependencySnapshot {
        repository: "example/api".into(),
        inventory: Some(DependencyInventory {
            observed_at: now,
            packages: vec![],
            direct_deps: Some(0),
            transitive_deps: None,
        }),
        security: Some(DependencySecurity {
            observed_at: now,
            alerts: vec![],
        }),
    };
    value.validate()?;
    let package = DependencyPackage {
        id: "pkg".into(),
        name: "name".into(),
        version: None,
        license: None,
        purl: None,
    };
    if let Some(inventory) = &mut value.inventory {
        inventory.packages = vec![package; 10_001];
    }
    assert!(value.validate().is_err());
    value.inventory = None;
    let alert: GithubDependencyAlert = serde_json::from_value(
        serde_json::to_value(snapshot()?)?["security"]["alerts"][0].clone(),
    )?;
    if let Some(security) = &mut value.security {
        security.alerts = vec![alert; 10_001];
    }
    assert!(value.validate().is_err());
    Ok(())
}
