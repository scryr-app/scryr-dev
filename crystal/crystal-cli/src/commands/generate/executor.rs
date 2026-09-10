//! Manifest execution backend selection.

use crate::args::GenerateRequest;
use crate::manifest_python::{
    ManifestPythonEnvironment, ManifestPythonMode, prepare_manifest_python_environment,
    run_manifest_python_command,
};
use crate::scryr_dir::resolve_scryr_dir;
use crate::uv::ensure_managed_uv;
use std::path::Path;

/// Selected manifest execution backend.
pub(crate) enum ManifestExecutor {
    /// Local uv-managed Python environment.
    Local(ManifestPythonEnvironment),
}

impl ManifestExecutor {
    /// Run formatting, lint, or type checks without executing user code.
    pub(crate) fn tool(&self, root: &Path, file: &Path, mode: &str) -> Result<String, String> {
        match self {
            Self::Local(environment) => {
                crate::manifest_python::adapter::run_manifest_tool(root, file, mode, environment)
            }
        }
    }

    /// Run the manifest adapter in the selected backend.
    pub(crate) fn run(
        &self,
        manifest_dir: &Path,
        manifest_file: &Path,
        mode: ManifestPythonMode,
    ) -> Result<String, String> {
        match self {
            Self::Local(environment) => {
                run_manifest_python_command(manifest_dir, manifest_file, mode, environment)
            }
        }
    }
}

/// Prepare the requested manifest execution backend.
pub(crate) fn prepare_manifest_executor(
    args: &GenerateRequest,
    manifest_dir: &Path,
) -> Result<ManifestExecutor, String> {
    prepare_environment(args, manifest_dir).map(ManifestExecutor::Local)
}

/// Install uv and sync the Scryr-managed Python environment for manifest execution.
fn prepare_environment(
    args: &GenerateRequest,
    manifest_dir: &Path,
) -> Result<ManifestPythonEnvironment, String> {
    let scryr_dir = resolve_scryr_dir(args.scryr_dir.as_deref())?;
    let managed_uv = ensure_managed_uv(&scryr_dir)?;
    let python_environment = ManifestPythonEnvironment::from_manifest_dir(
        &scryr_dir,
        manifest_dir,
        managed_uv.executable(),
    );
    prepare_manifest_python_environment(manifest_dir, &python_environment)?;
    Ok(python_environment)
}
