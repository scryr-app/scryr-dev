//! Python manifest adapter execution and environment preparation.

use super::embedded_sdk::{file_url, materialize_embedded_sdk_package};
use super::environment::ManifestPythonEnvironment;
use super::output::{ManifestPythonMode, PythonAdapterOutput, format_manifest_adapter_output};
use super::uv_runner::run_uv_command;
use std::ffi::OsString;
use std::fs;
use std::path::Path;

/// Python module that executes manifest code and reports Python-only data.
pub(super) const MANIFEST_PYTHON_MODULE: &str = "scryr.cli";
/// Python version Scryr manages internally for manifest execution.
const MANAGED_PYTHON_VERSION: &str = "3.14";
/// Execute the Python manifest adapter for one manifest file.
pub(crate) fn run_manifest_python_command(
    manifest_dir: &Path,
    manifest_file: &Path,
    mode: ManifestPythonMode,
    environment: &ManifestPythonEnvironment,
) -> Result<String, String> {
    let python_output = run_python_adapter(manifest_dir, manifest_file, mode, environment)?;
    format_manifest_adapter_output(manifest_file, mode, python_output)
}
/// Install or update the Scryr-managed Python virtual environment for manifest execution.
pub(crate) fn prepare_manifest_python_environment(
    manifest_dir: &Path,
    environment: &ManifestPythonEnvironment,
) -> Result<(), String> {
    if has_scryr_project(manifest_dir) && !manifest_dir.join("pyproject.toml").is_file() {
        return Err(format!(
            "{} opts into project dependencies, but pyproject.toml is missing from the same directory. Add a Python project with the scryr dependency and uv.lock, or remove scryr.toml to use the embedded SDK.",
            manifest_dir.join("scryr.toml").display()
        ));
    }
    ensure_managed_python_runtime(manifest_dir, environment)?;

    if !has_scryr_project(manifest_dir) {
        let sdk_package_dir = materialize_embedded_sdk_package(environment)?;
        return write_inline_adapter_script(environment, &sdk_package_dir);
    }

    let args = vec![
        OsString::from("sync"),
        OsString::from("--no-dev"),
        OsString::from("--locked"),
    ];
    let _stdout = run_uv_command(manifest_dir, args, environment)?;
    Ok(())
}

/// Ensure the Python interpreter used by Scryr is uv-managed and stored under `.scryr`.
fn ensure_managed_python_runtime(
    manifest_dir: &Path,
    environment: &ManifestPythonEnvironment,
) -> Result<(), String> {
    let _stdout = run_uv_command(
        manifest_dir,
        [
            OsString::from("python"),
            OsString::from("install"),
            OsString::from(MANAGED_PYTHON_VERSION),
            OsString::from("--managed-python"),
            OsString::from("--install-dir"),
            environment.python_install_path.as_os_str().to_os_string(),
        ],
        environment,
    )?;
    Ok(())
}

/// Execute the Python adapter and parse its stdout as JSON.
fn run_python_adapter(
    manifest_dir: &Path,
    manifest_file: &Path,
    mode: ManifestPythonMode,
    environment: &ManifestPythonEnvironment,
) -> Result<PythonAdapterOutput, String> {
    let args = if has_scryr_project(manifest_dir) {
        vec![
            OsString::from("run"),
            OsString::from("--no-dev"),
            OsString::from("--no-sync"),
            OsString::from("python"),
            OsString::from("-m"),
            OsString::from(MANIFEST_PYTHON_MODULE),
            manifest_file.as_os_str().to_os_string(),
            OsString::from(mode.as_flag()),
        ]
    } else {
        vec![
            OsString::from("run"),
            OsString::from("--no-project"),
            environment.inline_adapter_path().into_os_string(),
            manifest_file.as_os_str().to_os_string(),
            OsString::from(mode.as_flag()),
        ]
    };
    let stdout = run_uv_command(manifest_dir, args, environment)?;
    PythonAdapterOutput::parse(manifest_file, mode, &stdout)
}

/// Return whether this manifest directory explicitly opts into project dependencies.
pub(super) fn has_scryr_project(manifest_dir: &Path) -> bool {
    manifest_dir.join("scryr.toml").is_file()
}

/// Write a PEP 723 inline-metadata adapter for repos without a Scryr project.
fn write_inline_adapter_script(
    environment: &ManifestPythonEnvironment,
    sdk_package_dir: &Path,
) -> Result<(), String> {
    let sdk_package_url = file_url(sdk_package_dir);
    let inline_adapter = format!(
        r#"# /// script
# requires-python = ">=3.14"
# dependencies = [
#   "scryr @ {sdk_package_url}",
# ]
# ///
from scryr.cli import main

raise SystemExit(main())
"#
    );

    let path = environment.inline_adapter_path();
    if path.is_file() && fs::read_to_string(&path).is_ok_and(|existing| existing == inline_adapter)
    {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&path, inline_adapter)
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))
}
