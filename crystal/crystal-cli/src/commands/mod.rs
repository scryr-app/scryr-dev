//! Top-level CLI command dispatch.

mod auth;
mod generate;
mod serve;

use crate::args::ResolvedCommand;

/// Execute a fully resolved top-level command.
pub(crate) async fn run(command: ResolvedCommand) -> Result<(), String> {
    match command {
        ResolvedCommand::Serve(args) => serve::run(&args).await,
        ResolvedCommand::Generate(args) => generate::run(&args).await,
        ResolvedCommand::Auth(args) => auth::run(&args).await,
    }
}
