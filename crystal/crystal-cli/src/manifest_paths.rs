use std::path::{Path, PathBuf};

/// Resolve a manifest module path against the manifest project directory.
pub(crate) fn resolve_manifest_file(
    manifest_dir: &Path,
    manifest_file: &Path,
) -> Result<PathBuf, String> {
    let candidate = if manifest_file.is_absolute() {
        manifest_file.to_path_buf()
    } else {
        manifest_dir.join(manifest_file)
    };
    let resolved = candidate.canonicalize().map_err(|e| {
        format!(
            "Failed to resolve manifest file {}: {e}",
            candidate.display()
        )
    })?;

    if !resolved.is_file() {
        return Err(format!(
            "Manifest path is not a file: {}",
            resolved.display()
        ));
    }
    if !resolved
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "py" || ext == "scry")
    {
        return Err(format!(
            "Manifest path must point to a Python or .scry file: {}",
            resolved.display()
        ));
    }

    Ok(resolved)
}

/// Derive the persisted artifact key for a manifest module.
pub(crate) fn derive_manifest_artifact_key(manifest_dir: &Path, manifest_file: &Path) -> String {
    let relative = manifest_file
        .strip_prefix(manifest_dir)
        .unwrap_or(manifest_file);

    let components = relative.components().collect::<Vec<_>>();
    let sample_dir = match components.as_slice() {
        [tests_root, samples_root, sample_dir, file_name]
            if tests_root.as_os_str() == "tests"
                && samples_root.as_os_str() == "samples"
                && file_name.as_os_str() == "index.scry" =>
        {
            Some(sample_dir)
        }
        _ => None,
    };

    if let Some(sample_dir) = sample_dir {
        return sample_dir.as_os_str().to_string_lossy().into_owned();
    }

    relative
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{derive_manifest_artifact_key, resolve_manifest_file};
    use std::path::Path;

    #[test]
    fn resolve_manifest_file_accepts_relative_python_paths() -> Result<(), String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let resolved = resolve_manifest_file(
            &manifest_dir,
            Path::new("tests/samples/open_saas/index.scry"),
        )?;

        assert!(resolved.ends_with("tests/samples/open_saas/index.scry"));
        Ok(())
    }

    #[test]
    fn resolve_manifest_file_accepts_scry_paths() -> Result<(), String> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|error| format!("failed to create temp dir: {error}"))?;
        let manifest_file = temp_dir.path().join("main.scry");
        std::fs::write(&manifest_file, "from scryr import Manifest\n")
            .map_err(|error| format!("failed to write manifest file: {error}"))?;

        let resolved = resolve_manifest_file(temp_dir.path(), Path::new("main.scry"))?;

        assert_eq!(
            resolved,
            manifest_file
                .canonicalize()
                .map_err(|error| format!("failed to canonicalize manifest file: {error}"))?
        );
        Ok(())
    }

    #[test]
    fn derive_manifest_artifact_key_uses_sample_directory_name() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifest");
        let manifest_file = manifest_dir.join("tests/samples/open_saas/index.scry");

        let key = derive_manifest_artifact_key(&manifest_dir, &manifest_file);

        assert_eq!(key, "open_saas");
    }

    #[test]
    fn derive_manifest_artifact_key_does_not_special_case_top_level_samples() {
        let manifest_dir = Path::new("/workspace/manifest");
        let manifest_file = manifest_dir.join("samples/open_saas/index.scry");

        let key = derive_manifest_artifact_key(manifest_dir, &manifest_file);

        assert_eq!(key, "samples/open_saas/index");
    }

    #[test]
    fn derive_manifest_artifact_key_accepts_external_manifest_files() {
        let manifest_dir = Path::new("/workspace/scryr/manifest");
        let manifest_file = Path::new("/third-party/acme/index.scry");

        let key = derive_manifest_artifact_key(manifest_dir, manifest_file);

        assert_eq!(key, "third-party/acme/index");
    }
}
