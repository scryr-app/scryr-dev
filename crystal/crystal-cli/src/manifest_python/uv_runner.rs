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

    let cleaned = cleaned.trim();
    let location = python_error_location(cleaned);
    if cleaned.contains("PydanticSerializationError")
        && cleaned.contains("Circular reference detected")
    {
        let location = location
            .map(|value| format!(" at {value}"))
            .unwrap_or_default();
        return format!(
            "Manifest serialization failed{location}: a circular reference was found.\n\n\
			Check `connections` for cycles or nested full Manifest objects. Use\n\
			name-only references such as `Manifest(name=\"API\")` in connections,\n\
			and avoid self-references.\n\nDetails: Circular reference detected while serializing the manifest."
        );
    }

    location.map_or_else(
        || cleaned.to_string(),
        |location| format!("Manifest error at {location}:\n\n{cleaned}"),
    )
}

/// Extract the last Python source location from a traceback.
fn python_error_location(stderr: &str) -> Option<String> {
    stderr.lines().rev().find_map(|line| {
        let file_start = line.find("File \"")? + "File \"".len();
        let file_end = line[file_start..].find('"')? + file_start;
        let line_start = line[file_end..].find("line ")? + file_end + "line ".len();
        let line_number = line[line_start..]
            .split(|character: char| !character.is_ascii_digit())
            .next()
            .filter(|value| !value.is_empty())?;
        Some(format!("{}:{}", &line[file_start..file_end], line_number))
    })
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
    use super::{pretty_uv_stderr, python_error_location, run_uv_command};
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
    fn python_error_location_extracts_manifest_line() {
        assert_eq!(
            python_error_location("Traceback\n  File \"index.scry\", line 17, in <module>"),
            Some("index.scry:17".to_string())
        );
    }

    #[test]
    fn python_error_location_uses_the_last_traceback_frame() {
        assert_eq!(
            python_error_location(
                "Traceback\n  File \"scryr/runtime.py\", line 40, in run\n  File \"index.scry\", line 29, in <module>",
            ),
            Some("index.scry:29".to_string())
        );
    }

    #[test]
    fn pretty_uv_stderr_preserves_errors_without_source_locations() {
        assert_eq!(
            pretty_uv_stderr("ValidationError: invalid manifest"),
            "ValidationError: invalid manifest"
        );
    }

    #[test]
    fn pretty_uv_stderr_adds_location_to_regular_manifest_errors() {
        let error = pretty_uv_stderr(
            "Traceback (most recent call last):\n  File \"/project/index.scry\", line 8, in <module>\nValueError: invalid name",
        );
        assert!(error.starts_with("Manifest error at /project/index.scry:8:"));
        assert!(error.contains("ValueError: invalid name"));
    }

    #[test]
    fn pretty_uv_stderr_explains_circular_manifest_references() {
        let error = pretty_uv_stderr(
            "Traceback...\n  File \"index.scry\", line 42, in <module>\nPydanticSerializationError: Error serializing to JSON: ValueError: Circular reference detected (id repeated)",
        );
        assert!(error.contains("Manifest serialization failed at index.scry:42"));
        assert!(error.contains("name-only references"));
        assert!(!error.contains("Traceback"));
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
        assert!(error.contains("Manifest error at index.scry:1"));
        assert!(error.contains("Traceback (most recent call last):\n  File \"index.scry\""));
        assert!(error.contains("ValueError: bad manifest"));
        assert!(!error.contains("\\x1b[31m"));
        Ok(())
    }
}
