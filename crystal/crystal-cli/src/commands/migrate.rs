//! Database schema migration command.

/// Ensure the configured database has the current schema.
pub(super) async fn run() -> Result<(), String> {
    let pool = crystal_core::persistence::connect_from_env().await?;
    crystal_core::persistence::ensure_table(&pool).await?;
    println!("Database schema is up to date");
    Ok(())
}
