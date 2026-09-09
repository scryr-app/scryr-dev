//! Top-level CLI command dispatch.

mod auth;
mod generate;
mod report;
mod serve;

use crate::args::ResolvedCommand;

/// Execute a fully resolved top-level command.
pub(crate) async fn run(command: ResolvedCommand) -> Result<(), String> {
    match command {
        ResolvedCommand::ReportActionStatus(args) => report::run(&args).await,
        ResolvedCommand::Migrate => {
            let pool = crystal_core::persistence::connect_from_env().await?;
            crystal_core::persistence::ensure_table(&pool).await?;
            println!("Database schema is up to date");
            Ok(())
        }
        ResolvedCommand::Serve(args) => serve::run(&args).await,
        ResolvedCommand::Generate(args) => generate::run(&args).await,
        ResolvedCommand::Auth(args) => auth::run(&args).await,
    }
}
