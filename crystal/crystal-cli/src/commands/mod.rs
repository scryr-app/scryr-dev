//! Top-level CLI command dispatch.

mod auth;
mod generate;
mod migrate;
mod observations;
mod report;
mod serve;

use crate::args::ResolvedCommand;

/// Execute a fully resolved top-level command.
pub(crate) async fn run(command: ResolvedCommand) -> Result<(), String> {
    match command {
        ResolvedCommand::Report(args) => observations::run(&args).await,
        ResolvedCommand::ReportActionStatus(args) => report::run(&args).await,
        ResolvedCommand::Migrate => migrate::run().await,
        ResolvedCommand::Serve(args) => serve::run(&args).await,
        ResolvedCommand::Generate(args) => generate::run(&args).await,
        ResolvedCommand::Auth(args) => auth::run(&args).await,
    }
}
