//! Generate command argument parsing and request resolution.

use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Shared arguments for generate subcommands.
#[derive(ClapArgs, Debug, Clone)]
pub(crate) struct GenerateCommonArgs {
    /// Manifest Python or `.scry` file to execute, relative to `manifest_dir` or absolute.
    #[arg(long = "path", default_value = "index.scry")]
    pub(crate) manifest_file: PathBuf,
    /// Path to the Python manifest project directory.
    #[arg(long, default_value = ".")]
    pub(crate) manifest_dir: PathBuf,
    /// Directory for Scryr-managed local state, including the uv Python environment.
    #[arg(long)]
    pub(crate) scryr_dir: Option<PathBuf>,
    /// GraphQL endpoint used by `push`.
    #[arg(long = "endpoint", alias = "graphql-url", env = "SCRYR_ENDPOINT")]
    pub(crate) graphql_url: Option<String>,
    /// Clerk organization id to use for generated manifest uploads.
    #[arg(long, env = "SCRYR_CLERK_ORG_ID")]
    pub(crate) clerk_org_id: Option<String>,
    /// Git commit SHA to associate with generated manifest uploads.
    #[arg(long, env = "SCRYR_GIT_COMMIT_SHA")]
    pub(crate) git_commit_sha: Option<String>,
}

/// Shared arguments for Forge-backed generate subcommands.
#[derive(ClapArgs, Debug, Clone)]
pub(crate) struct GenerateForgeArgs {
    #[command(flatten)]
    pub(crate) common: GenerateCommonArgs,
    /// Forge name or variable to use.
    #[arg(long)]
    pub(crate) forge: Option<String>,
}

/// Resolved generation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenerateOutput {
    /// Persist generated map artifacts through GraphQL.
    Upload,
    /// Print manifest field type metadata as JSON.
    Types,
    /// Print the manifest model JSON schema.
    Schema,
    /// Print generated artifact JSON for release embedding.
    ArtifactJson,
    /// Print the selected Forge as mise.toml.
    Mise,
    /// Print Docker Compose YAML.
    Compose,
    /// Print a devcontainer.json.
    Devcontainer,
}

/// Fully resolved generate request consumed by command execution.
#[derive(Debug, Clone)]
pub(crate) struct GenerateRequest {
    /// Requested output target.
    pub(crate) output: GenerateOutput,
    /// Manifest Python or `.scry` file to execute.
    pub(crate) manifest_file: PathBuf,
    /// Path to the Python manifest project directory.
    pub(crate) manifest_dir: PathBuf,
    /// Directory for Scryr-managed local state.
    pub(crate) scryr_dir: Option<PathBuf>,
    /// Forge selector for Forge-backed targets.
    pub(crate) forge: Option<String>,
    /// GraphQL endpoint used by upload.
    pub(crate) graphql_url: Option<String>,
    /// Clerk organization id to use for generated manifest uploads.
    pub(crate) clerk_org_id: Option<String>,
    /// Git commit SHA to associate with generated manifest uploads.
    pub(crate) git_commit_sha: Option<String>,
}

/// Convert command-specific options into a concrete request.
pub(crate) fn resolve_generate_request(
    output: GenerateOutput,
    common: GenerateCommonArgs,
    forge: Option<String>,
) -> GenerateRequest {
    GenerateRequest {
        output,
        manifest_file: common.manifest_file,
        manifest_dir: common.manifest_dir,
        scryr_dir: common.scryr_dir,
        forge,
        graphql_url: common
            .graphql_url
            .or_else(|| std::env::var("SCRYR_GRAPHQL_URL").ok()),
        clerk_org_id: common.clerk_org_id,
        git_commit_sha: common.git_commit_sha,
    }
}
