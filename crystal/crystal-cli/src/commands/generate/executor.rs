//! Manifest execution backend selection.

use crate::args::GenerateRequest;
use crate::manifest_python::{
    ManifestPythonEnvironment, ManifestPythonMode, prepare_manifest_python_environment,
    run_manifest_python_command,
};
use crate::manifest_sprite::{SpriteExecutionEnvironment, run_manifest_sprite_command};
use crate::scryr_dir::resolve_scryr_dir;
use crate::uv::ensure_managed_uv;
use std::path::Path;

/// Selected manifest execution backend.
pub(super) enum ManifestExecutor {
    /// Local uv-managed Python environment.
    Local(ManifestPythonEnvironment),
    /// Remote sprites.dev execution environment.
    Sprite(SpriteExecutionEnvironment),
}

impl ManifestExecutor {
    /// Run the manifest adapter in the selected backend.
    pub(super) fn run(
        &self,
        manifest_dir: &Path,
        manifest_file: &Path,
        mode: ManifestPythonMode,
    ) -> Result<String, String> {
        match self {
            Self::Local(environment) => {
                run_manifest_python_command(manifest_dir, manifest_file, mode, environment)
            }
            Self::Sprite(environment) => {
                run_manifest_sprite_command(manifest_dir, manifest_file, mode, environment)
            }
        }
    }
}

/// Prepare the requested manifest execution backend.
pub(super) fn prepare_manifest_executor(
    args: &GenerateRequest,
    manifest_dir: &Path,
) -> Result<ManifestExecutor, String> {
    if let Some(sprite_name) = &args.sprite {
        return Ok(ManifestExecutor::Sprite(SpriteExecutionEnvironment::new(
            args.sprite_bin.clone(),
            sprite_name.clone(),
            args.sprite_org.clone(),
        )));
    }

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
