//! Local coverage of organization sample initialization and concurrent requests.
#[path = "support/samples.rs"]
mod contract;

use crystal_core::persistence::{DatabasePool, connect_sqlite_path, seed_organization_samples};

#[tokio::test]
async fn organization_samples_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    contract::exercise(DatabasePool::Sqlite(pool)).await
}

#[tokio::test]
async fn concurrent_requests_seed_once() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let pool = connect_sqlite_path(&temp.path().join("samples.db")).await?;
    let second_pool = connect_sqlite_path(&temp.path().join("samples.db")).await?;
    let third_pool = connect_sqlite_path(&temp.path().join("samples.db")).await?;
    let context = contract::context("concurrent_org");
    let samples = contract::samples()?;
    let (first, second, third) = tokio::join!(
        seed_organization_samples(&second_pool, &context, &samples),
        seed_organization_samples(&third_pool, &context, &samples),
        seed_organization_samples(&pool, &context, &samples),
    );
    first?;
    second?;
    third?;
    assert_eq!(
        contract::count(&pool, "organization_sample_seeds").await?,
        1
    );
    assert_eq!(contract::count(&pool, "generated_manifests").await?, 4);
    assert_eq!(
        contract::count(&pool, "generated_manifest_uploads").await?,
        4
    );
    Ok(())
}

#[tokio::test]
async fn existing_sample_filename_preserves_the_entire_source()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::SqlitePool::connect("sqlite::memory:").await?;
    let pool = DatabasePool::Sqlite(sqlite);
    let context = contract::context("existing_source");
    let samples = contract::samples()?;
    let mut existing = samples[0].clone();
    existing.scry_identifier = Some("my_diagram".into());
    crystal_core::persistence::persist_generated_manifest(&pool, &existing, &context).await?;
    seed_organization_samples(&pool, &context, &samples).await?;
    let maps =
        crystal_core::persistence::list_generated_manifest_maps(&pool, &context.clerk_org_id)
            .await?;
    assert_eq!(maps.len(), 3);
    assert!(maps.iter().any(|map| map.scry_identifier == "my_diagram"));
    assert!(
        !maps
            .iter()
            .any(|map| map.scry_identifier.starts_with("first_"))
    );
    Ok(())
}
