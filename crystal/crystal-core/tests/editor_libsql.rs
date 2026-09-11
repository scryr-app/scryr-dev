//! Remote libSQL contract test using a dedicated, disposable test database.
//! Run with `--ignored` and set `SCRYR_TEST_LIBSQL_URL` and `SCRYR_TEST_LIBSQL_TOKEN`.
//! Keep the embedded engine disabled: sqlx already links the local `SQLite` engine.
#[path = "support/editor.rs"]
mod contract;

#[tokio::test]
#[ignore = "requires a dedicated remote libSQL test database"]
async fn libsql_document_round_trip() -> Result<(), String> {
    let url = std::env::var("SCRYR_TEST_LIBSQL_URL").map_err(|e| e.to_string())?;
    let token = std::env::var("SCRYR_TEST_LIBSQL_TOKEN").map_err(|e| e.to_string())?;
    let database = libsql::Builder::new_remote(url, token)
        .build()
        .await
        .map_err(|e| e.to_string())?;
    contract::exercise(crystal_core::persistence::DatabasePool::Turso(
        std::sync::Arc::new(database),
    ))
    .await
}
