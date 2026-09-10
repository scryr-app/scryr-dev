//! Local development and standalone server arguments.
use super::GenerateCommonArgs;
use clap::Args;

/// Start the server and optionally load a local manifest.
#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Independent clap switches, with conflicts enforced by the parser.
pub(crate) struct LocalServeArgs {
    /// HTTP server configuration.
    #[command(flatten)]
    pub(crate) server: crystal_server::ServerArgs,
    /// Manifest execution configuration.
    #[command(flatten)]
    pub(crate) common: GenerateCommonArgs,
    /// Run only the server, without reading or changing local manifest files.
    #[arg(long, conflicts_with_all = ["watch", "no_format", "no_open"])]
    pub(crate) server_only: bool,
    /// Reload after manifest source changes.
    #[arg(long)]
    pub(crate) watch: bool,
    /// Check formatting without rewriting source files.
    #[arg(long)]
    pub(crate) no_format: bool,
    /// Do not open the browser automatically.
    #[arg(long)]
    pub(crate) no_open: bool,
}
impl std::ops::Deref for LocalServeArgs {
    type Target = crystal_server::ServerArgs;
    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

/// CLI server options, including local manifest loading.
pub(crate) type ServerArgs = LocalServeArgs;
