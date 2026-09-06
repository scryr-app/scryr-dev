//! uv command execution helpers.

use super::environment::ManifestPythonEnvironment;
use std::ffi::OsString;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::process::Command;

/// Execute a `uv` command inside the Python manifest directory and return stdout text.
pub(super) fn run_uv_command<I>(
    manifest_dir: &Path,
    args: I,
    environment: &ManifestPythonEnvironment,
) -> Result<String, String>
where
    I: IntoIterator<Item = OsString>,
{
    let output = Command::new(&environment.uv_executable)
        .args(args)
        .current_dir(manifest_dir)
        .env("UV_CACHE_DIR", &environment.cache_path)
        .env("UV_MANAGED_PYTHON", "true")
        .env("UV_PYTHON_DOWNLOADS", "automatic")
        .env("UV_PYTHON_INSTALL_BIN", "false")
        .env("UV_PYTHON_INSTALL_DIR", &environment.python_install_path)
        .env("UV_PROJECT_ENVIRONMENT", &environment.venv_path)
        .output()
        .map_err(|e| format!("Failed to spawn uv: {e}"))?;

    if output.status.success() {
        return String::from_utf8(output.stdout)
            .map_err(|e| format!("uv produced invalid UTF-8 in stdout: {e}"));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "uv command failed in {}: {stderr}",
        manifest_dir.display()
    ))
}

/// Resolve a Python interpreter already provisioned for the manifest project.
#[cfg(test)]
pub(super) fn project_python_executable(manifest_dir: &Path) -> Option<PathBuf> {
    let venv_python = venv_python_executable(&manifest_dir.join(".venv"));
    if venv_python.is_file() {
        return Some(venv_python);
    }

    mise_python_executable(manifest_dir)
}

/// Return the conventional Python executable inside a virtual environment.
#[cfg(test)]
pub(super) fn venv_python_executable(venv_path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        venv_path.join("Scripts").join("python.exe")
    }

    #[cfg(not(windows))]
    {
        venv_path.join("bin").join("python")
    }
}

/// Ask mise for the project Python when it is available.
#[cfg(test)]
fn mise_python_executable(manifest_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("mise")
        .args(["which", "python"])
        .current_dir(manifest_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let path = PathBuf::from(stdout.trim());
    if path.is_file() { Some(path) } else { None }
}
