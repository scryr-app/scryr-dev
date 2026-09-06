//! Managed `uv` version verification.

use super::REQUIRED_UV_VERSION;
use super::paths::ManagedUv;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Load and verify the managed `uv` executable.
pub(super) fn load_managed_uv(executable: PathBuf) -> Result<ManagedUv, String> {
    let version = uv_version(&executable)?;
    let expected_prefix = format!("uv {REQUIRED_UV_VERSION}");
    if !version.starts_with(&expected_prefix) {
        return Err(format!(
            "{} is {version}; Scryr requires {expected_prefix}",
            executable.display()
        ));
    }

    Ok(ManagedUv { executable })
}

/// Return the version string for a `uv` executable.
fn uv_version(executable: &Path) -> Result<String, String> {
    let output = Command::new(executable)
        .arg("--version")
        .output()
        .map_err(|error| {
            format!(
                "Failed to execute {} --version: {error}",
                executable.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{} --version failed: {stderr}",
            executable.display()
        ));
    }

    String::from_utf8(output.stdout)
        .map(|version| version.trim().to_string())
        .map_err(|error| {
            format!(
                "{} --version produced invalid UTF-8: {error}",
                executable.display()
            )
        })
}
