//! Remote shell script generation for Sprite execution.

use super::bundle::SpriteExecutionPlan;
use super::remote_paths::shell_single_quote;
use crate::manifest_python::ManifestPythonMode;

/// Remote shell script that installs uv when needed and runs the manifest adapter.
pub(super) fn remote_execution_script(
    execution_plan: &SpriteExecutionPlan,
    mode: ManifestPythonMode,
) -> String {
    format!(
        "set -eu\n\
         rm -rf {run_root}\n\
         mkdir -p {run_root}\n\
         tar -xf {bundle_path} -C {run_root}\n\
         if ! command -v uv >/dev/null 2>&1; then\n\
           curl -LsSf https://astral.sh/uv/install.sh | sh >/dev/null\n\
           export PATH=\"$HOME/.local/bin:$PATH\"\n\
         fi\n\
         cd {package_dir}\n\
         uv run --no-dev python -m scryr.cli {manifest_file} {mode_flag}",
        run_root = shell_single_quote(&execution_plan.remote_run_root),
        bundle_path = shell_single_quote(&execution_plan.remote_bundle_path),
        package_dir = shell_single_quote(&execution_plan.remote_scryr_package_dir),
        manifest_file = shell_single_quote(&execution_plan.remote_manifest_file),
        mode_flag = mode.as_flag(),
    )
}

#[cfg(test)]
mod tests {
    use super::remote_execution_script;
    use crate::manifest_python::ManifestPythonMode;
    use crate::manifest_sprite::bundle::SpriteExecutionPlan;
    use std::path::Path;

    #[test]
    fn remote_execution_script_runs_scryr_adapter() {
        let plan = SpriteExecutionPlan {
            bundle_path: Path::new("/tmp/scryr-cli-run.tar").to_path_buf(),
            remote_bundle_path: "/tmp/scryr-cli-run.tar".to_string(),
            remote_run_root: "/tmp/scryr-cli/run".to_string(),
            remote_scryr_package_dir: "/tmp/scryr-cli/run/sdk/scryr".to_string(),
            remote_manifest_file: "/tmp/scryr-cli/run/manifest/index.scry".to_string(),
        };

        let script = remote_execution_script(&plan, ManifestPythonMode::Json);

        assert!(script.contains("tar -xf"));
        assert!(script.contains("uv run --no-dev python -m scryr.cli"));
        assert!(script.contains("--json"));
    }
}
