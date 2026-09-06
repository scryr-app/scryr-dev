//! Shared API behavior independent of the CLI and GraphQL server.

use crystal_core::{Error, generation, persistence};

#[test]
fn invalid_generation_input_returns_validation_error() {
    let result = generation::render_compose_yaml("not JSON", None);
    assert!(matches!(result, Err(Error::Validation(_))));
    let result = generation::map_artifacts_from_manifest_json("not JSON", "sample");
    assert!(matches!(result, Err(Error::Validation(_))));
}

#[tokio::test]
async fn unavailable_storage_returns_storage_error() -> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::SqlitePool::connect("sqlite::memory:").await?;
    sqlite.close().await;
    let result = persistence::ensure_table(&persistence::DatabasePool::Sqlite(sqlite)).await;
    assert!(matches!(result, Err(Error::Storage(_))));
    Ok(())
}
