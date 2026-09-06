//! Managed `uv` installer behavior.

use super::REQUIRED_UV_VERSION;
use super::paths::managed_uv_path;
use super::version::load_managed_uv;
use reqwest::blocking::Client;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

/// Hidden development/test override for the Astral installer script source.
const UV_INSTALLER_ENV: &str = "SCRYR_UV_INSTALLER";

/// Ensure Scryr's pinned `uv` executable is installed under `<scryr-dir>/bin/uv`.
pub(crate) fn ensure_managed_uv(scryr_dir: &Path) -> Result<super::ManagedUv, String> {
    let installer_override = env::var_os(UV_INSTALLER_ENV).map(PathBuf::from);
    ensure_managed_uv_with_installer(scryr_dir, installer_override.as_deref())
}

/// Ensure Scryr's pinned `uv` executable with an optional installer script override.
fn ensure_managed_uv_with_installer(
    scryr_dir: &Path,
    installer_override: Option<&Path>,
) -> Result<super::ManagedUv, String> {
    let managed_uv_path = managed_uv_path(scryr_dir);

    if let Ok(managed_uv) = load_managed_uv(managed_uv_path.clone()) {
        return Ok(managed_uv);
    }

    install_uv_from_astral(scryr_dir, installer_override)?;
    load_managed_uv(managed_uv_path)
}

/// Install `uv` using Astral's standalone unmanaged installer.
fn install_uv_from_astral(
    scryr_dir: &Path,
    installer_override: Option<&Path>,
) -> Result<(), String> {
    let bin_dir = scryr_dir.join("bin");
    fs::create_dir_all(&bin_dir)
        .map_err(|error| format!("Failed to create {}: {error}", bin_dir.display()))?;

    let installer = installer_script(installer_override)?;
    run_installer_script(&installer, &bin_dir)
}

/// Return the installer script, either from the hidden test override or Astral.
fn installer_script(installer_override: Option<&Path>) -> Result<String, String> {
    if let Some(path) = installer_override {
        return fs::read_to_string(path).map_err(|error| {
            format!(
                "Failed to read uv installer override {}: {error}",
                path.display()
            )
        });
    }

    let url = installer_url();
    download_installer_script(url)
}

/// Download the installer without dropping reqwest's blocking runtime inside Tokio.
fn download_installer_script(url: String) -> Result<String, String> {
    thread::spawn(move || {
        Client::new()
            .get(&url)
            .send()
            .map_err(|error| format!("Failed to download Scryr-managed uv from {url}: {error}"))?
            .error_for_status()
            .map_err(|error| {
                format!("Scryr-managed uv installer request failed for {url}: {error}")
            })?
            .text()
            .map_err(|error| {
                format!("Failed to read Scryr-managed uv installer from {url}: {error}")
            })
    })
    .join()
    .map_err(|panic| {
        let message = panic
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("unknown panic");
        format!("Failed to join Scryr-managed uv installer download thread: {message}")
    })?
}

/// Return the Astral standalone installer URL for the pinned `uv` version.
fn installer_url() -> String {
    if cfg!(windows) {
        format!("https://astral.sh/uv/{REQUIRED_UV_VERSION}/install.ps1")
    } else {
        format!("https://astral.sh/uv/{REQUIRED_UV_VERSION}/install.sh")
    }
}

/// Run an installer script without modifying shell profiles or user-level `uv` state.
fn run_installer_script(installer: &str, bin_dir: &Path) -> Result<(), String> {
    if cfg!(windows) {
        run_windows_installer(installer, bin_dir)
    } else {
        run_unix_installer(installer, bin_dir)
    }
}

/// Run the Unix installer through `sh`.
#[cfg(not(windows))]
fn run_unix_installer(installer: &str, bin_dir: &Path) -> Result<(), String> {
    let mut child = Command::new("sh")
        .env("UV_UNMANAGED_INSTALL", bin_dir)
        .env("UV_NO_MODIFY_PATH", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to spawn sh for Scryr-managed uv installer: {error}"))?;

    child
        .stdin
        .as_mut()
        .ok_or_else(|| "Failed to open uv installer stdin".to_string())?
        .write_all(installer.as_bytes())
        .map_err(|error| format!("Failed to send uv installer script to sh: {error}"))?;

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Failed to wait for Scryr-managed uv installer: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "Scryr-managed uv installer failed for {}: {stderr}",
        bin_dir.display()
    ))
}

/// Run the Windows installer through PowerShell.
#[cfg(windows)]
fn run_windows_installer(installer: &str, bin_dir: &Path) -> Result<(), String> {
    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "-"])
        .env("UV_UNMANAGED_INSTALL", bin_dir)
        .env("UV_NO_MODIFY_PATH", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!("Failed to spawn PowerShell for Scryr-managed uv installer: {error}")
        })?;

    child
        .stdin
        .as_mut()
        .ok_or_else(|| "Failed to open uv installer stdin".to_string())?
        .write_all(installer.as_bytes())
        .map_err(|error| format!("Failed to send uv installer script to PowerShell: {error}"))?;

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Failed to wait for Scryr-managed uv installer: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "Scryr-managed uv installer failed for {}: {stderr}",
        bin_dir.display()
    ))
}

#[cfg(windows)]
/// Placeholder for the Unix installer path when compiling on Windows.
fn run_unix_installer(_installer: &str, _bin_dir: &Path) -> Result<(), String> {
    unreachable!("Unix installer is not used on Windows")
}

#[cfg(not(windows))]
/// Placeholder for the Windows installer path when compiling on Unix.
fn run_windows_installer(_installer: &str, _bin_dir: &Path) -> Result<(), String> {
    unreachable!("Windows installer is not used on Unix")
}

#[cfg(test)]
mod tests {
    use super::{ensure_managed_uv_with_installer, installer_url};
    use crate::uv::REQUIRED_UV_VERSION;
    use crate::uv::paths::managed_uv_path;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn installer_url_uses_pinned_astral_version() {
        let url = installer_url();
        assert!(url.contains("https://astral.sh/uv/"));
        assert!(url.contains(REQUIRED_UV_VERSION));
    }

    #[cfg(unix)]
    #[test]
    fn ensure_managed_uv_installs_from_override_without_path_lookup() -> Result<(), String> {
        let temp_dir =
            TempDir::new().map_err(|error| format!("failed to create temp dir: {error}"))?;
        let installer_path = temp_dir.path().join("install.sh");
        fs::write(
            &installer_path,
            format!(
                r#"#!/bin/sh
set -eu
mkdir -p "$UV_UNMANAGED_INSTALL"
cat > "$UV_UNMANAGED_INSTALL/uv" <<'EOF'
#!/bin/sh
if [ "$1" = "--version" ]; then
  printf 'uv {REQUIRED_UV_VERSION}\n'
fi
EOF
chmod +x "$UV_UNMANAGED_INSTALL/uv"
"#
            ),
        )
        .map_err(|error| format!("failed to write fake installer: {error}"))?;

        let managed_uv = ensure_managed_uv_with_installer(temp_dir.path(), Some(&installer_path))?;
        assert_eq!(managed_uv.executable(), managed_uv_path(temp_dir.path()));
        Ok(())
    }
}
