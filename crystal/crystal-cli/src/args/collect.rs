//! Local collector management from typed index.scry declarations.
#![allow(clippy::missing_docs_in_private_items)]
use super::GenerateCommonArgs;
use clap::{Args, Subcommand};
#[derive(Args, Debug, Clone)]
pub(crate) struct CollectArgs {
    #[command(subcommand)]
    pub command: CollectCommand,
}
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum CollectCommand {
    /// List concrete collectors and their typed execution settings.
    List(CollectOptions),
    /// Check installed integration tools, versions, and credentials.
    Doctor(CollectOptions),
    /// Run declared collectors; use the active local owner when one exists.
    Run(CollectOptions),
    /// Show local collector lifecycle, freshness policy and scheduling.
    Status(CollectOptions),
    /// Pause schedules and cancel active child processes.
    Pause(CollectOptions),
    /// Resume the active local owner's schedules.
    Resume(CollectOptions),
}
#[derive(Args, Debug, Clone)]
pub(crate) struct CollectOptions {
    #[command(flatten)]
    pub common: GenerateCommonArgs,
    /// Select a stable `manifest_id`.
    #[arg(long)]
    pub manifest: Option<String>,
    #[arg(long,value_parser=["repository","checks","metrics","tests","dependencies","performance"])]
    pub section: Option<String>,
    /// Select a collector ID (defaults to its integration name when unspecified).
    #[arg(long)]
    pub collector: Option<String>,
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}
