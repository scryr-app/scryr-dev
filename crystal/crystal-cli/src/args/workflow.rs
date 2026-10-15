//! User-facing manifest workflows.
#![allow(clippy::missing_docs_in_private_items)]
use super::generate::GenerateForgeArgs;
use super::{GenerateCommonArgs, GenerateOutput, GenerateRequest, resolve_generate_request};
use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub(crate) struct FormatArgs {
    #[command(flatten)]
    pub common: GenerateCommonArgs,
    /// Check formatting without writing files.
    #[arg(long)]
    pub check: bool,
}
#[derive(Args, Debug, Clone)]
pub(crate) struct LintArgs {
    #[command(flatten)]
    pub common: GenerateCommonArgs,
    /// Apply safe lint fixes.
    #[arg(long)]
    pub fix: bool,
}
#[derive(Args, Debug, Clone)]
pub(crate) struct ExportArgs {
    #[command(subcommand)]
    pub target: ExportTarget,
}
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ExportTarget {
    /// Print validated diagram JSON.
    Json(GenerateCommonArgs),
    /// Print mise.toml for a Forge.
    Mise(GenerateForgeArgs),
    /// Print Docker Compose YAML for a Forge.
    Compose(GenerateForgeArgs),
    /// Print devcontainer.json for a Forge.
    Devcontainer(GenerateForgeArgs),
}
impl ExportArgs {
    pub(crate) fn request(self) -> GenerateRequest {
        match self.target {
            ExportTarget::Json(a) => {
                resolve_generate_request(GenerateOutput::ArtifactJson, a, None)
            }
            ExportTarget::Mise(a) => {
                resolve_generate_request(GenerateOutput::Mise, a.common, a.forge)
            }
            ExportTarget::Compose(a) => {
                resolve_generate_request(GenerateOutput::Compose, a.common, a.forge)
            }
            ExportTarget::Devcontainer(a) => {
                resolve_generate_request(GenerateOutput::Devcontainer, a.common, a.forge)
            }
        }
    }
}
#[derive(Args, Debug, Clone)]
pub(crate) struct InspectArgs {
    #[command(subcommand)]
    pub target: InspectTarget,
}
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum InspectTarget {
    /// Print runtime type metadata (does not statically check types).
    Types(GenerateCommonArgs),
    /// Print the SDK JSON schema.
    Schema(GenerateCommonArgs),
}
impl InspectArgs {
    pub(crate) fn request(self) -> GenerateRequest {
        match self.target {
            InspectTarget::Types(a) => resolve_generate_request(GenerateOutput::Types, a, None),
            InspectTarget::Schema(a) => resolve_generate_request(GenerateOutput::Schema, a, None),
        }
    }
}
#[derive(Args, Debug, Clone)]
pub(crate) struct QueryArgs {
    #[command(flatten)]
    pub common: GenerateCommonArgs,
    /// Query name from a metrics or analytics declaration.
    #[arg(required_unless_present = "list", conflicts_with = "list")]
    pub name: Option<String>,
    /// List available queries without contacting a provider.
    #[arg(long)]
    pub list: bool,
    /// Select a public manifest variable, name, or manifest ID.
    #[arg(long)]
    pub manifest: Option<String>,
    /// Restrict selection when a manifest uses the same query name in both providers.
    #[arg(long, value_parser = ["prometheus", "posthog"])]
    pub provider: Option<String>,
    /// Print machine-readable results.
    #[arg(long)]
    pub json: bool,
}

/// Source selection for reporters, with explicit-ID compatibility mode.
#[derive(Args, Debug, Clone)]
pub(crate) struct ReportSourceArgs {
    /// Manifest entrypoint (defaults to index.scry when no explicit ID is supplied).
    #[arg(long)]
    pub path: Option<std::path::PathBuf>,
    #[arg(long, default_value = ".")]
    pub manifest_dir: std::path::PathBuf,
    #[arg(long)]
    pub scryr_dir: Option<std::path::PathBuf>,
    /// Public variable, manifest name, or stable manifest ID.
    #[arg(long)]
    pub manifest: Option<String>,
}

/// Legacy endpoint fallback; clap applies explicit flags and `SCRYR_ENDPOINT` first.
pub(crate) fn default_endpoint() -> String {
    std::env::var("SCRYR_GRAPHQL_URL").unwrap_or_else(|_| {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8000);
        format!("http://127.0.0.1:{port}/graphql")
    })
}
