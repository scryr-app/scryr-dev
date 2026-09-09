//! Operational reporter command arguments.
#![allow(clippy::missing_docs_in_private_items)]
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Clone, Debug, Args)]
pub(crate) struct ReportsArgs {
    #[command(subcommand)]
    pub command: ReportCommand,
}
#[derive(Clone, Debug, Subcommand)]
pub(crate) enum ReportCommand {
    /// Report a GitHub workflow event (compatible with `report-action-status`).
    Actions(super::ReportArgs),
    /// Import `JUnit` XML results from one complete suite/shard.
    Tests(ObservationArgs),
    /// Import Cobertura XML or LCOV line coverage.
    Coverage(ObservationArgs),
    /// Collect a complete Dependabot snapshot, or import an offline JSON array.
    Dependencies(ObservationArgs),
    /// Record a deployment for one environment.
    Deployment(ObservationArgs),
}
#[derive(Clone, Debug, Args)]
pub(crate) struct ObservationArgs {
    #[arg(long)]
    pub manifest_id: String,
    #[arg(
        long,
        env = "SCRYR_ENDPOINT",
        default_value = "http://127.0.0.1:8000/graphql"
    )]
    pub endpoint: String,
    #[arg(long, env = "SCRYR_CLERK_ORG_ID")]
    pub clerk_org_id: Option<String>,
    #[arg(long)]
    pub file: Vec<PathBuf>,
    #[arg(long)]
    pub format: Option<String>,
    #[arg(long, alias = "scope", default_value = "default")]
    pub suite: String,
    #[arg(long)]
    pub shard: Option<String>,
    #[arg(long, env = "GITHUB_RUN_ID")]
    pub run_id: String,
    #[arg(long, env = "GITHUB_RUN_ATTEMPT", default_value_t = 1)]
    pub attempt: u32,
    #[arg(long, env = "GITHUB_SHA")]
    pub commit_sha: Option<String>,
    #[arg(long, env = "GITHUB_REF_NAME")]
    pub branch: Option<String>,
    /// RFC3339 source completion/snapshot time; required for retry-stable delivery.
    #[arg(long)]
    pub observed_at: String,
    #[arg(long)]
    pub report_url: Option<String>,
    #[arg(long)]
    pub repository: Option<String>,
    #[arg(long)]
    pub manifest_path: Option<String>,
    #[arg(long, default_value = "dependabot")]
    pub provider: String,
    #[arg(long)]
    pub environment: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}
