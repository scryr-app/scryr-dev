//! Generate command argument parsing and request resolution.

use clap::{Args as ClapArgs, Subcommand};
use std::path::PathBuf;

/// Arguments for manifest generation.
#[derive(ClapArgs, Debug, Clone)]
#[command(
    after_help = "Examples:\n  scryr generate upload --path index.scry\n  scryr generate types --path index.scry\n  scryr generate schema --path index.scry\n  scryr generate mise --path index.scry --forge \"MERN Forge\"\n  scryr generate compose --path index.scry --forge \"MERN Forge\"\n  scryr generate devcontainer --path index.scry --forge \"MERN Forge\""
)]
pub(crate) struct GenerateArgs {
    /// Output target to generate.
    #[command(subcommand)]
    pub(crate) target: GenerateTarget,
}

/// Generate subcommands.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum GenerateTarget {
    /// Persist generated map artifacts through GraphQL.
    Upload(GenerateCommonArgs),
    /// Print manifest field type metadata as JSON.
    Types(GenerateCommonArgs),
    /// Print the manifest model JSON schema.
    Schema(GenerateCommonArgs),
    /// Print generated artifact JSON for release embedding.
    #[command(name = "artifact-json", hide = true)]
    ArtifactJson(GenerateCommonArgs),
    /// Print the selected Forge as mise.toml.
    Mise(GenerateForgeArgs),
    /// Print Docker Compose YAML for service tools in the selected Forge.
    Compose(GenerateForgeArgs),
    /// Print a devcontainer.json for the selected Forge.
    Devcontainer(GenerateForgeArgs),
}

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
    /// GraphQL endpoint used by `generate upload`.
    #[arg(long, env = "SCRYR_GRAPHQL_URL")]
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

impl GenerateArgs {
    /// Resolve raw generate CLI args into the command execution request.
    pub(crate) fn into_request(self) -> GenerateRequest {
        match self.target {
            GenerateTarget::Upload(common) => {
                resolve_generate_request(GenerateOutput::Upload, common, None)
            }
            GenerateTarget::Types(common) => {
                resolve_generate_request(GenerateOutput::Types, common, None)
            }
            GenerateTarget::Schema(common) => {
                resolve_generate_request(GenerateOutput::Schema, common, None)
            }
            GenerateTarget::ArtifactJson(common) => {
                resolve_generate_request(GenerateOutput::ArtifactJson, common, None)
            }
            GenerateTarget::Mise(args) => {
                resolve_generate_request(GenerateOutput::Mise, args.common, args.forge)
            }
            GenerateTarget::Compose(args) => {
                resolve_generate_request(GenerateOutput::Compose, args.common, args.forge)
            }
            GenerateTarget::Devcontainer(args) => {
                resolve_generate_request(GenerateOutput::Devcontainer, args.common, args.forge)
            }
        }
    }
}

/// Convert command-specific options into a concrete request.
fn resolve_generate_request(
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
        graphql_url: common.graphql_url,
        clerk_org_id: common.clerk_org_id,
        git_commit_sha: common.git_commit_sha,
    }
}
