//! Bundle planning and staging for remote Sprite execution.

use super::remote_paths::{relative_file_name, relative_path_text, remote_path_join};
use crate::manifest_source::{manifest_source_files_in_directory, manifest_source_root};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

/// Files and remote paths needed for one Sprite execution.
#[derive(Debug)]
pub(super) struct SpriteExecutionPlan {
    /// Local bundle archive uploaded before execution.
    pub(super) bundle_path: PathBuf,
    /// Remote bundle archive path.
    pub(super) remote_bundle_path: String,
    /// Remote path to the staged run root.
    pub(super) remote_run_root: String,
    /// Remote path to the staged Scryr Python package.
    pub(super) remote_scryr_package_dir: String,
    /// Remote path to the manifest file to execute.
    pub(super) remote_manifest_file: String,
}

impl SpriteExecutionPlan {
    /// Build a fresh remote staging plan for one manifest.
    pub(super) fn build(manifest_dir: &Path, manifest_file: &Path) -> Result<Self, String> {
        let run_id = remote_run_id(manifest_dir, manifest_file);
        let remote_run_root = format!("/tmp/scryr-cli/{run_id}");
        let remote_sdk_root = format!("{remote_run_root}/sdk");
        let remote_manifest_root = format!("{remote_run_root}/manifest");
        let remote_bundle_path = format!("/tmp/scryr-cli-{run_id}.tar");

        let scryr_package_dir = scryr_package_dir(manifest_dir)?;
        let remote_scryr_package_dir =
            remote_path_join(&remote_sdk_root, relative_file_name(&scryr_package_dir)?);
        let bundle_root = local_bundle_root(&run_id);
        let bundle_sdk_root = bundle_root.join("sdk");
        let bundle_scryr_package_dir =
            bundle_sdk_root.join(relative_file_name(&scryr_package_dir)?);
        let bundle_manifest_root = bundle_root.join("manifest");

        let mut staged_files =
            collect_scryr_package_files(&scryr_package_dir, &bundle_scryr_package_dir)?;

        let source_root = manifest_source_root(manifest_file)?;
        let source_files = manifest_source_files_in_directory(&source_root)?;
        for path in source_files {
            let relative = path.strip_prefix(&source_root).map_err(|error| {
                format!(
                    "Failed to make manifest source file {} relative to {}: {error}",
                    path.display(),
                    source_root.display()
                )
            })?;
            let bundle_path = bundle_manifest_root.join(relative);
            staged_files.push(StagedFile {
                local: path,
                bundle_path,
            });
        }

        let relative_manifest = manifest_file.strip_prefix(&source_root).map_err(|error| {
            format!(
                "Failed to make manifest file {} relative to {}: {error}",
                manifest_file.display(),
                source_root.display()
            )
        })?;
        let remote_manifest_file = remote_path_join(
            &remote_manifest_root,
            &relative_path_text(relative_manifest)?,
        );

        staged_files.sort_by(|left, right| left.bundle_path.cmp(&right.bundle_path));
        staged_files.dedup_by(|left, right| left.bundle_path == right.bundle_path);
        stage_bundle_files(&staged_files)?;
        let bundle_path = create_bundle_archive(&run_id, &bundle_root)?;

        Ok(Self {
            bundle_path,
            remote_bundle_path,
            remote_run_root,
            remote_scryr_package_dir,
            remote_manifest_file,
        })
    }
}

/// One local source file staged into the upload bundle.
#[derive(Debug)]
struct StagedFile {
    /// Local source path.
    local: PathBuf,
    /// Path inside the local bundle staging directory.
    bundle_path: PathBuf,
}

/// Resolve the local Scryr Python package directory.
fn scryr_package_dir(manifest_dir: &Path) -> Result<PathBuf, String> {
    let workspace_member = manifest_dir.join("scryr");
    if workspace_member.join("pyproject.toml").is_file() {
        return Ok(workspace_member);
    }
    if manifest_dir.join("pyproject.toml").is_file() && manifest_dir.join("src/scryr").is_dir() {
        return Ok(manifest_dir.to_path_buf());
    }
    Err(format!(
        "Could not find the Scryr Python package under {}. Expected scryr/pyproject.toml or src/scryr.",
        manifest_dir.display()
    ))
}

/// Collect files required to install and run the Scryr Python package remotely.
fn collect_scryr_package_files(
    package_dir: &Path,
    bundle_package_dir: &Path,
) -> Result<Vec<StagedFile>, String> {
    let mut staged_files = Vec::new();
    collect_staged_files(
        package_dir,
        package_dir,
        bundle_package_dir,
        &mut staged_files,
    )?;
    staged_files.retain(|file| is_scryr_package_upload(package_dir, &file.local));
    Ok(staged_files)
}

/// Recursively collect package files to stage.
fn collect_staged_files(
    root: &Path,
    current: &Path,
    bundle_root: &Path,
    staged_files: &mut Vec<StagedFile>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("Failed to read directory {}: {error}", current.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to read directory entry in {}: {error}",
                current.display()
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "Failed to read file type for {}: {error}",
                entry.path().display()
            )
        })?;

        if file_type.is_dir() {
            if should_skip_upload_dir(&path) {
                continue;
            }
            collect_staged_files(root, &path, bundle_root, staged_files)?;
        } else if file_type.is_file() {
            let relative = path.strip_prefix(root).map_err(|error| {
                format!(
                    "Failed to make Scryr package file {} relative to {}: {error}",
                    path.display(),
                    root.display()
                )
            })?;
            let bundle_path = bundle_root.join(relative);
            staged_files.push(StagedFile {
                local: path,
                bundle_path,
            });
        }
    }
    Ok(())
}

/// Copy all selected files into the local bundle staging tree.
fn stage_bundle_files(staged_files: &[StagedFile]) -> Result<(), String> {
    for file in staged_files {
        let parent = file.bundle_path.parent().ok_or_else(|| {
            format!(
                "Bundle path has no parent directory: {}",
                file.bundle_path.display()
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
        fs::copy(&file.local, &file.bundle_path).map_err(|error| {
            format!(
                "Failed to stage {} at {}: {error}",
                file.local.display(),
                file.bundle_path.display()
            )
        })?;
    }
    Ok(())
}

/// Create a tar archive from the local bundle staging tree.
fn create_bundle_archive(run_id: &str, bundle_root: &Path) -> Result<PathBuf, String> {
    let bundle_path = env::temp_dir().join(format!("scryr-cli-{run_id}.tar"));
    let output = Command::new("tar")
        .args([
            OsString::from("-cf"),
            bundle_path.as_os_str().to_os_string(),
            OsString::from("-C"),
            bundle_root.as_os_str().to_os_string(),
            OsString::from("."),
        ])
        .output()
        .map_err(|error| format!("Failed to create Sprite upload bundle with tar: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "tar failed while creating Sprite upload bundle: {stderr}"
        ));
    }

    Ok(bundle_path)
}

/// Return the local staging directory for one Sprite bundle.
fn local_bundle_root(run_id: &str) -> PathBuf {
    env::temp_dir().join(format!("scryr-cli-bundle-{run_id}"))
}

/// Return whether a package source path should be staged remotely.
fn is_scryr_package_upload(package_dir: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(package_dir) else {
        return false;
    };
    matches!(
        relative.to_str(),
        Some("pyproject.toml" | "README.md" | "LICENSE" | "LICENSE.md")
    ) || relative.starts_with("src/scryr")
}

/// Return whether a directory should not be uploaded to a Sprite.
fn should_skip_upload_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    name.starts_with('.')
        || matches!(
            name,
            "__pycache__" | "node_modules" | "uv-cache" | "target" | "dist" | "build"
        )
}

/// Return a fresh remote run id for one local manifest execution.
fn remote_run_id(manifest_dir: &Path, manifest_file: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(manifest_dir.to_string_lossy().as_bytes());
    hasher.update(b"\0");
    hasher.update(manifest_file.to_string_lossy().as_bytes());
    if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        hasher.update(b"\0");
        hasher.update(duration.as_nanos().to_string().as_bytes());
    }
    let digest = hasher.finalize();
    digest
        .iter()
        .take(12)
        .fold(String::with_capacity(24), |mut output, byte| {
            const HEX: &[u8; 16] = b"0123456789abcdef";
            let byte = *byte;
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
            output
        })
}

#[cfg(test)]
mod tests {
    use super::{SpriteExecutionPlan, scryr_package_dir};
    use std::path::Path;

    #[test]
    fn scryr_package_dir_finds_workspace_member() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");

        let package_dir = scryr_package_dir(&manifest_dir)?;

        assert!(package_dir.ends_with("manifest/scryr"));
        Ok(())
    }

    #[test]
    fn sprite_execution_plan_bundles_sdk_and_manifest_sources() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let manifest_file = manifest_dir.join("tests/samples/open_saas/index.scry");

        let plan = SpriteExecutionPlan::build(&manifest_dir, &manifest_file)?;

        assert!(plan.bundle_path.is_file());
        assert!(plan.remote_bundle_path.starts_with("/tmp/scryr-cli-"));
        assert!(
            Path::new(&plan.remote_bundle_path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("tar"))
        );
        assert!(plan.remote_manifest_file.ends_with("/manifest/index.scry"));
        Ok(())
    }
}
