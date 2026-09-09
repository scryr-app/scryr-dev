//! Arguments for the native GitHub Actions reporter.
#![allow(clippy::missing_docs_in_private_items)]
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub(crate) struct ReportArgs {
    #[arg(long)]
    pub manifest_id: String,
    #[arg(long, env = "GITHUB_EVENT_PATH")]
    pub event_file: PathBuf,
    #[arg(long, env = "SCRYR_ENDPOINT")]
    pub endpoint: String,
    #[arg(long, env = "SCRYR_CLERK_ORG_ID")]
    pub clerk_org_id: Option<String>,
    /// Override automatic main/master/repository-default branch selection.
    #[arg(long)]
    pub branch: Option<String>,
    /// Optionally restrict reporting to one workflow.
    #[arg(long)]
    pub workflow_id: Option<u64>,
}
