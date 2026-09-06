//! Managed Python environment path derivation.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Environment paths used by uv for Scryr-managed Python execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestPythonEnvironment {
    /// Root directory for Scryr-managed execution state for one manifest project.
    pub(super) root_path: PathBuf,
    /// Managed uv executable used to sync and run Python.
    pub(super) uv_executable: PathBuf,
    /// Fully managed virtual environment path passed to uv.
    pub(super) venv_path: PathBuf,
    /// uv cache path kept alongside Scryr-managed state.
    pub(super) cache_path: PathBuf,
    /// uv-managed Python installation directory owned by Scryr.
    pub(super) python_install_path: PathBuf,
}

impl ManifestPythonEnvironment {
    /// Build the managed Python environment isolated for one manifest project.
    pub(crate) fn from_manifest_dir(
        scryr_dir: &Path,
        manifest_dir: &Path,
        uv_executable: &Path,
    ) -> Self {
        let environment_root = isolated_environment_root(scryr_dir, manifest_dir);
        Self {
            root_path: environment_root.clone(),
            uv_executable: uv_executable.to_path_buf(),
            venv_path: environment_root.join(".venv"),
            cache_path: environment_root.join("uv-cache"),
            python_install_path: scryr_dir.join("python"),
        }
    }

    /// Adapter script path used when a manifest repo does not opt into a Scryr project.
    pub(super) fn inline_adapter_path(&self) -> PathBuf {
        self.root_path.join("scryr_adapter.py")
    }
}
/// Return a deterministic Scryr state subdirectory for one manifest project.
pub(super) fn isolated_environment_root(scryr_dir: &Path, manifest_dir: &Path) -> PathBuf {
    let manifest_dir_text = manifest_dir.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(manifest_dir_text.as_bytes());
    let digest = hasher.finalize();
    let hash = hex_digest_prefix(&digest, 8);
    let name = manifest_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(sanitize_environment_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "manifest".to_string());

    scryr_dir.join("python-envs").join(format!("{name}-{hash}"))
}

/// Return the leading bytes of a digest as lower-case hexadecimal text.
pub(super) fn hex_digest_prefix(digest: &[u8], byte_count: usize) -> String {
    digest.iter().take(byte_count).fold(
        String::with_capacity(byte_count * 2),
        |mut output, byte| {
            const HEX: &[u8; 16] = b"0123456789abcdef";
            let byte = *byte;
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
            output
        },
    )
}

/// Sanitize path text for use in local state directory names.
fn sanitize_environment_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect()
}
