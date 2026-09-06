//! Embedded Scryr Python SDK materialization.

use super::environment::{ManifestPythonEnvironment, hex_digest_prefix};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Directory name prefix for the Scryr Python SDK package materialized for inline uv scripts.
const SDK_PACKAGE_DIR_PREFIX: &str = "scryr-sdk";

/// One file included in the embedded Scryr Python SDK package.
struct EmbeddedSdkFile {
    /// Relative path inside the materialized SDK package.
    path: &'static str,
    /// File content compiled into the CLI binary.
    content: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/embedded_scryr_sdk.rs"));
/// Write the embedded Scryr Python SDK package under Scryr-managed local state.
pub(super) fn materialize_embedded_sdk_package(
    environment: &ManifestPythonEnvironment,
) -> Result<PathBuf, String> {
    let package_dir = environment.root_path.join(format!(
        "{SDK_PACKAGE_DIR_PREFIX}-{}",
        embedded_sdk_digest()
    ));
    for file in EMBEDDED_SDK_FILES {
        write_embedded_sdk_file(&package_dir, file)?;
    }
    Ok(package_dir)
}

/// Return a short content digest for the embedded SDK package.
fn embedded_sdk_digest() -> String {
    let mut hasher = Sha256::new();
    for file in EMBEDDED_SDK_FILES {
        hasher.update(file.path.as_bytes());
        hasher.update([0]);
        hasher.update(file.content.as_bytes());
        hasher.update([0]);
    }
    hex_digest_prefix(&hasher.finalize(), 12)
}

/// Write one embedded SDK file, skipping unchanged files.
fn write_embedded_sdk_file(package_dir: &Path, file: &EmbeddedSdkFile) -> Result<(), String> {
    let path = package_dir.join(file.path);
    if path.is_file() && fs::read_to_string(&path).is_ok_and(|existing| existing == file.content) {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&path, file.content)
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))
}

/// Return a file URL suitable for a PEP 508 direct path dependency.
pub(super) fn file_url(path: &Path) -> String {
    let mut path_text = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) && !path_text.starts_with('/') {
        path_text = format!("/{path_text}");
    }
    format!("file://{path_text}")
}
