//! Manifest source root detection.

use std::path::{Path, PathBuf};

/// Return the source root that should be staged with a manifest file.
pub(crate) fn manifest_source_root(manifest_file: &Path) -> Result<PathBuf, String> {
    sample_directory(manifest_file)
        .or_else(|| manifest_file.parent().map(Path::to_path_buf))
        .ok_or_else(|| {
            format!(
                "Manifest file path has no parent directory: {}",
                manifest_file.display()
            )
        })
}

/// Return the sample directory when the manifest path follows `samples/name/index.scry`.
fn sample_directory(manifest_file: &Path) -> Option<PathBuf> {
    let file_name = manifest_file.file_name()?.to_str()?;
    if file_name != "index.scry" {
        return None;
    }
    let parent = manifest_file.parent()?;
    let samples_dir = parent.parent()?;

    if samples_dir.file_name()? == "samples" {
        Some(parent.to_path_buf())
    } else {
        None
    }
}
