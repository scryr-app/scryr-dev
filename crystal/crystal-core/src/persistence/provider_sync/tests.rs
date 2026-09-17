//! Collection status storage, authorization, and provider isolation contracts.

use super::*;

fn principal(org: &str) -> ManifestRequestContext {
    ManifestRequestContext {
        clerk_user_id: "user".into(),
        clerk_org_id: org.into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:admin".into()),
        clerk_org_permissions: vec![],
    }
}

async fn exercise(pool: &DatabasePool) -> Result<(), Box<dyn std::error::Error>> {
    let context = principal(&uuid::Uuid::new_v4().to_string());
    let org = &context.clerk_org_id;
    assert!(
        read_provider_sync(pool, org, "services/api")
            .await?
            .is_empty()
    );
    assert!(
        record_provider_sync(
            pool,
            &context,
            "services/api",
            "github",
            Some("offline".into())
        )
        .await?
    );
    let failed = read_provider_sync(pool, org, "services/api").await?;
    assert_eq!(failed["github"].error.as_deref(), Some("offline"));
    assert_eq!(failed["github"].last_success_at, None);
    assert!(record_provider_sync(pool, &context, "services/api", "github", None).await?);
    let successful = read_provider_sync(pool, org, "services/api").await?;
    assert!(successful["github"].error.is_none());
    assert_eq!(
        successful["github"].last_success_at,
        Some(successful["github"].last_attempt_at)
    );
    assert!(successful["github"].last_attempt_at >= failed["github"].last_attempt_at);
    assert!(
        record_provider_sync(
            pool,
            &context,
            "services/api",
            "github",
            Some("rate limited".into())
        )
        .await?
    );
    let failed_again = read_provider_sync(pool, org, "services/api").await?;
    assert_eq!(
        failed_again["github"].last_success_at,
        successful["github"].last_success_at
    );
    assert_eq!(
        failed_again["github"].error.as_deref(),
        Some("rate limited")
    );
    assert!(failed_again["github"].last_attempt_at >= successful["github"].last_attempt_at);
    assert!(record_provider_sync(pool, &context, "services/api", "another_provider", None).await?);
    let providers = read_provider_sync(pool, org, "services/api").await?;
    assert_eq!(providers.len(), 2);
    assert_eq!(providers["github"], failed_again["github"]);
    assert!(
        read_provider_sync(pool, "other_org", "services/api")
            .await?
            .is_empty()
    );
    assert!(
        read_provider_sync(pool, org, "services/web")
            .await?
            .is_empty()
    );
    assert!(record_provider_sync(pool, &context, "services/api", "github", None).await?);
    assert!(
        read_provider_sync(pool, org, "services/api").await?["github"]
            .error
            .is_none()
    );
    assert_rejected_writes(pool, &context).await?;
    Ok(())
}

async fn assert_rejected_writes(
    pool: &DatabasePool,
    context: &ManifestRequestContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut denied = context.clone();
    denied.clerk_org_role = None;
    assert!(
        record_provider_sync(pool, &denied, "services/api", "github", None)
            .await
            .is_err()
    );
    assert!(
        record_provider_sync(pool, &principal(""), "services/api", "github", None)
            .await
            .is_err()
    );
    assert!(read_provider_sync(pool, "", "services/api").await.is_err());
    for provider in ["", "GitHub", "bad/provider", &"a".repeat(65)] {
        assert!(
            record_provider_sync(pool, context, "services/api", provider, None)
                .await
                .is_err()
        );
    }
    assert!(
        record_provider_sync(pool, context, "", "github", None)
            .await
            .is_err()
    );
    assert!(
        record_provider_sync(
            pool,
            context,
            "services/api",
            "github",
            Some("x".repeat(4097))
        )
        .await
        .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn collection_status_lifecycle_and_isolation() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    exercise(&DatabasePool::Sqlite(sqlite)).await
}

#[tokio::test]
async fn collection_status_survives_reopening_storage() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("status.db");
    let pool = crate::persistence::connect_sqlite_path(&path).await?;
    record_provider_sync(&pool, &principal("org"), "api", "github", None).await?;
    let before = read_provider_sync(&pool, "org", "api").await?;
    if let DatabasePool::Sqlite(sqlite) = pool {
        sqlite.close().await;
    }
    let reopened = crate::persistence::connect_sqlite_path(&path).await?;
    assert_eq!(read_provider_sync(&reopened, "org", "api").await?, before);
    Ok(())
}

#[tokio::test]
#[ignore = "requires an empty, dedicated remote libSQL test database"]
async fn libsql_collection_status_contract() -> Result<(), Box<dyn std::error::Error>> {
    let database = libsql::Builder::new_remote(
        std::env::var("SCRYR_TEST_LIBSQL_URL")?,
        std::env::var("SCRYR_TEST_LIBSQL_TOKEN")?,
    )
    .build()
    .await?;
    exercise(&DatabasePool::Turso(std::sync::Arc::new(database))).await
}
