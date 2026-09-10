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

    let stderr = pretty_uv_stderr(&String::from_utf8_lossy(&output.stderr));
    Err(format!(
        "uv command failed in {}:\n\n{stderr}",
        manifest_dir.display()
    ))
}

/// Make uv/Python diagnostics readable when surfaced by the CLI.
fn pretty_uv_stderr(stderr: &str) -> String {
    let mut cleaned = String::with_capacity(stderr.len());
    let mut in_escape = false;

    for character in stderr.chars() {
        if in_escape {
            if character.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }

        if character == '\u{1b}' {
            in_escape = true;
        } else {
            cleaned.push(character);
        }
    }

    cleaned.trim().to_string()
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

#[cfg(test)]
mod tests {
    use super::{pretty_uv_stderr, run_uv_command};
    use crate::manifest_python::environment::ManifestPythonEnvironment;
    use std::error::Error;
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn pretty_uv_stderr_trims_and_removes_terminal_styles() {
        assert_eq!(pretty_uv_stderr("\u{1b}[31merror\u{1b}[0m\n\n"), "error");
    }

    #[test]
    fn failing_uv_traceback_is_returned_as_readable_multiline_error() -> Result<(), Box<dyn Error>>
    {
        let temp_dir = tempdir()?;
        let uv_path = temp_dir.path().join("uv");
        fs::write(
            &uv_path,
            "#!/bin/sh\nprintf '%s\\n' 'Traceback (most recent call last):' '  File \"index.scry\", line 1, in <module>' '    raise ValueError(\"bad manifest\")' 'ValueError: bad manifest' >&2\nexit 1\n",
        )?;
        fs::set_permissions(&uv_path, fs::Permissions::from_mode(0o755))?;

        let environment = ManifestPythonEnvironment::from_manifest_dir(
            temp_dir.path(),
            temp_dir.path(),
            &uv_path,
        );
        let Err(error) = run_uv_command(temp_dir.path(), [OsString::from("run")], &environment)
        else {
            return Err("fake uv unexpectedly succeeded".into());
        };

        assert!(error.contains("uv command failed"));
        assert!(error.contains("Traceback (most recent call last):\n  File \"index.scry\""));
        assert!(error.contains("ValueError: bad manifest"));
        assert!(!error.contains("\\x1b[31m"));
        Ok(())
    }
}
