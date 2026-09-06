//! Manifest source file collection.

use super::root::manifest_source_root;
use crystal_core::generated_manifest_envelope::ManifestSourceFile;
use std::fs;
use std::path::{Path, PathBuf};

/// Python-syntax Scryr manifest extension.
const SCRY_EXTENSION: &str = "scry";

/// Collect source files associated with the manifest artifact.
pub(crate) fn collect_manifest_source_files(
    manifest_file: &Path,
) -> Result<Vec<ManifestSourceFile>, String> {
    let root = manifest_source_root(manifest_file)?;
    let mut paths = manifest_source_files_in_directory(&root)?;
    paths.sort();
    let files = paths
        .into_iter()
        .map(|path| {
            let relative = path.strip_prefix(&root).map_err(|error| {
                format!(
                    "Failed to make manifest source file {} relative to {}: {error}",
                    path.display(),
                    root.display()
                )
            })?;
            source_file_from_path(&path, relative)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if files.is_empty() {
        return Err(format!(
            "No manifest source files found for {}",
            manifest_file.display()
        ));
    }

    Ok(files)
}

/// Recursively list manifest source files in a directory.
pub(crate) fn manifest_source_files_in_directory(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    collect_manifest_source_file_paths(root, &mut paths)?;
    Ok(paths)
}

/// Recursive implementation for `manifest_source_files_in_directory`.
fn collect_manifest_source_file_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root)
        .map_err(|error| format!("Failed to read directory {}: {error}", root.display()))?
    {
        let entry = entry
            .map_err(|error| format!("Failed to read entry in {}: {error}", root.display()))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "Failed to read file type for {}: {error}",
                entry.path().display()
            )
        })?;

        if file_type.is_dir() {
            if should_skip_manifest_source_dir(&path) {
                continue;
            }
            collect_manifest_source_file_paths(&path, paths)?;
        } else if file_type.is_file() && is_manifest_source_file(&path) {
            paths.push(path);
        }
    }
    Ok(())
}

/// Read a manifest source file into the manifest envelope shape.
fn source_file_from_path(path: &Path, relative_path: &Path) -> Result<ManifestSourceFile, String> {
    let content = fs::read_to_string(path).map_err(|error| {
        format!(
            "Failed to read manifest source file {}: {error}",
            path.display()
        )
    })?;
    Ok(ManifestSourceFile {
        path: relative_path.to_string_lossy().replace('\\', "/"),
        content,
    })
}

/// Return whether a path is an importable manifest source file.
fn is_manifest_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "py" || extension == SCRY_EXTENSION)
}

/// Return whether a directory should not be included in manifest source capture.
fn should_skip_manifest_source_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    name.starts_with('.')
        || matches!(
            name,
            "__pycache__" | "node_modules" | "uv-cache" | "target" | "dist"
        )
}

#[cfg(test)]
mod tests {
    use super::collect_manifest_source_files;
    use std::fs;
    use std::path::Path;

    #[test]
    fn sample_manifest_file_collection_uses_sample_directory() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let manifest_file = manifest_dir.join("samples/open_saas/index.scry");

        let files = collect_manifest_source_files(&manifest_file)?;

        assert!(files.iter().any(|file| file.path == "index.scry"));
        Ok(())
    }

    #[test]
    fn sample_manifest_file_collection_includes_scry_sources() -> Result<(), String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|error| format!("failed to create temp dir: {error}"))?;
        let sample_dir = temp_dir.path().join("samples/demo");
        fs::create_dir_all(&sample_dir)
            .map_err(|error| format!("failed to create sample dir: {error}"))?;
        let manifest_file = sample_dir.join("index.scry");
        fs::write(&manifest_file, "from scryr import Manifest\n")
            .map_err(|error| format!("failed to write manifest file: {error}"))?;
        fs::write(sample_dir.join("helper.py"), "VALUE = 1\n")
            .map_err(|error| format!("failed to write helper file: {error}"))?;

        let files = collect_manifest_source_files(&manifest_file)?;

        assert!(files.iter().any(|file| file.path == "index.scry"));
        assert!(files.iter().any(|file| file.path == "helper.py"));
        Ok(())
    }

    #[test]
    fn single_manifest_file_collection_includes_importable_siblings() -> Result<(), String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|error| format!("failed to create temp dir: {error}"))?;
        let manifest_file = temp_dir.path().join("main.scry");
        fs::write(&manifest_file, "from scryr import Manifest\n")
            .map_err(|error| format!("failed to write manifest file: {error}"))?;
        fs::write(temp_dir.path().join("helper.py"), "VALUE = 1\n")
            .map_err(|error| format!("failed to write helper file: {error}"))?;
        let hidden_venv = temp_dir.path().join(".venv/lib/site.py");
        let hidden_venv_parent = hidden_venv
            .parent()
            .ok_or_else(|| "hidden venv path should have a parent".to_string())?;
        fs::create_dir_all(hidden_venv_parent)
            .map_err(|error| format!("failed to create hidden venv dir: {error}"))?;
        fs::write(&hidden_venv, "IGNORED = 1\n")
            .map_err(|error| format!("failed to write hidden venv file: {error}"))?;

        let files = collect_manifest_source_files(&manifest_file)?;

        assert!(files.iter().any(|file| file.path == "main.scry"));
        assert!(files.iter().any(|file| file.path == "helper.py"));
        assert!(!files.iter().any(|file| file.path.contains(".venv")));
        Ok(())
    }
}
