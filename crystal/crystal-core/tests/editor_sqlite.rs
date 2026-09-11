//! `SQLite` source-editing contract.
#[path = "support/editor.rs"]
mod contract;

#[tokio::test]
async fn sqlite_document_round_trip() -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let pool = crystal_core::persistence::connect_sqlite_path(&dir.path().join("editor.db"))
        .await
        .map_err(|e| e.to_string())?;
    contract::exercise(pool).await
}
