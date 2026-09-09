//! Rendering for non-persistent generate targets.

use crate::args::{GenerateOutput, GenerateRequest};
use crate::commands::generate::executor::ManifestExecutor;
use crate::manifest_python::ManifestPythonMode;
use crystal_core::generation::{render_compose_yaml, render_devcontainer_json, render_mise_toml};
use std::path::Path;

/// Render a generate target that writes its result to stdout.
pub(super) fn render_stdout_target(
    output: GenerateOutput,
    args: &GenerateRequest,
    executor: &ManifestExecutor,
    manifest_dir: &Path,
    manifest_file: &Path,
) -> Result<bool, String> {
    let render_json = || executor.run(manifest_dir, manifest_file, ManifestPythonMode::Json);

    let rendered = match output {
        GenerateOutput::Types => {
            Some(executor.run(manifest_dir, manifest_file, ManifestPythonMode::Types)?)
        }
        GenerateOutput::Schema => {
            Some(executor.run(manifest_dir, manifest_file, ManifestPythonMode::Schema)?)
        }
        GenerateOutput::ArtifactJson => Some(render_json()?),
        GenerateOutput::Mise => Some(render_mise_toml(&render_json()?, args.forge.as_deref())?),
        GenerateOutput::Compose => {
            Some(render_compose_yaml(&render_json()?, args.forge.as_deref())?)
        }
        GenerateOutput::Devcontainer => Some(render_devcontainer_json(
            &render_json()?,
            args.forge.as_deref(),
        )?),
        GenerateOutput::Upload => None,
    };

    rendered.map_or(Ok(false), |content| {
        print!("{content}");
        Ok(true)
    })
}
