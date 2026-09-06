//! Remote sprites.dev manifest execution.
#![allow(clippy::redundant_pub_crate)]

mod bundle;
mod environment;
mod remote_paths;
mod runner;
mod script;

use crate::manifest_python::{
    ManifestPythonMode, PythonAdapterOutput, format_manifest_adapter_output,
};
use bundle::SpriteExecutionPlan;
pub(crate) use environment::SpriteExecutionEnvironment;
use runner::run_sprite_exec;
use std::path::Path;

/// Execute the Python manifest adapter inside a sprites.dev Sprite.
pub(crate) fn run_manifest_sprite_command(
    manifest_dir: &Path,
    manifest_file: &Path,
    mode: ManifestPythonMode,
    environment: &SpriteExecutionEnvironment,
) -> Result<String, String> {
    let execution_plan = SpriteExecutionPlan::build(manifest_dir, manifest_file)?;
    let stdout = run_sprite_exec(&execution_plan, mode, environment)?;
    let python_output = PythonAdapterOutput::parse(manifest_file, mode, &stdout)
        .map_err(|error| error.replace("Python manifest adapter", "Sprite manifest adapter"))?;
    format_manifest_adapter_output(manifest_file, mode, python_output)
}
