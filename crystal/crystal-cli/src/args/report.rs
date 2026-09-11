//! Arguments for the native GitHub Actions reporter.
#![allow(clippy::missing_docs_in_private_items)]
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub(crate) struct ReportArgs {
    /// Select a typed GitHub Actions pipeline by its source variable name.
    #[arg(long)]
    pub card: Option<String>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub source: super::ReportSourceArgs,
    #[arg(long, default_value = "")]
    pub manifest_id: String,
    #[arg(long, env = "GITHUB_EVENT_PATH")]
    pub event_file: PathBuf,
    #[arg(
        long,
        alias = "graphql-url",
        env = "SCRYR_ENDPOINT",
        default_value_t = super::workflow::default_endpoint()
    )]
    pub endpoint: String,
    #[arg(long, env = "SCRYR_CLERK_ORG_ID")]
    pub clerk_org_id: Option<String>,
    /// Override automatic main/master/repository-default branch selection.
    #[arg(long)]
    pub branch: Option<String>,
    /// Optionally restrict reporting to one workflow.
    #[arg(long)]
    pub workflow_id: Option<u64>,
    /// Complete GitHub jobs API response for this workflow run attempt.
    #[arg(long)]
    pub jobs_file: Option<PathBuf>,
}
