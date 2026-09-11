//! Separate executable: libSQL configures `SQLite` threading before initialization.
//! Running its initializer alongside sqlx unit tests introduces a global `SQLite` race.
#[path = "support/editor.rs"]
mod contract;

#[tokio::test]
async fn libsql_document_round_trip() -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let database = libsql::Builder::new_local(dir.path().join("editor.db"))
        .build()
        .await
        .map_err(|e| e.to_string())?;
    contract::exercise(crystal_core::persistence::DatabasePool::Turso(
        std::sync::Arc::new(database),
    ))
    .await
}
