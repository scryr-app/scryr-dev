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
