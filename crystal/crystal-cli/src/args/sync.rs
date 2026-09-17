//! One-shot collection using the same providers as the background scheduler.
use super::GenerateCommonArgs;
use clap::{Args, Subcommand};

/// Select a provider to synchronize.
#[derive(Args, Debug, Clone)]
pub(crate) struct SyncArgs {
    /// Provider command.
    #[command(subcommand)]
    pub command: SyncCommand,
}

/// Supported background providers.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SyncCommand {
    /// Collect declared GitHub Actions workflows using your existing gh login.
    Github(GithubSyncArgs),
}

/// Source and destination for a GitHub synchronization.
#[derive(Args, Debug, Clone)]
pub(crate) struct GithubSyncArgs {
    /// Manifest source and Scryr endpoint.
    #[command(flatten)]
    pub common: GenerateCommonArgs,
    /// Restrict collection to a public variable, name, or stable manifest ID.
    #[arg(long)]
    pub manifest: Option<String>,
    /// Print a machine-readable collection summary.
    #[arg(long)]
    pub json: bool,
}
