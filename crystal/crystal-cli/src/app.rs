#![allow(clippy::missing_docs_in_private_items)]

use crate::args::Args;
use crate::commands;
use clap::CommandFactory;

/// Execute the requested CLI command.
pub(crate) async fn run(args: &Args) -> Result<(), String> {
    if args.command.is_none() {
        Args::command()
            .print_help()
            .map_err(|error| format!("Failed to print help: {error}"))?;
        println!();
        return Ok(());
    }

    commands::run(args.resolved_command()?).await
}
