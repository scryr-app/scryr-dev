//! Run against an empty, disposable Turso database; never a production database.
#[path = "support/samples.rs"]
mod contract;

#[tokio::test]
#[ignore = "requires an empty, dedicated remote libSQL test database"]
async fn libsql_organization_samples_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let database = libsql::Builder::new_remote(
        std::env::var("SCRYR_TEST_LIBSQL_URL")?,
        std::env::var("SCRYR_TEST_LIBSQL_TOKEN")?,
    )
    .build()
    .await?;
    contract::exercise(crystal_core::persistence::DatabasePool::Turso(
        std::sync::Arc::new(database),
    ))
    .await
}
