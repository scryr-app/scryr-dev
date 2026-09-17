//! Local bounded artifacts, input hashes and watch matching.
#![allow(clippy::missing_docs_in_private_items)]
use crystal_core::collectors::CollectorConfig as C;
use crystal_core::collectors::{CollectorConfig, fingerprint};
use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::{
    collections::BTreeSet,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(super) fn private_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub(super) fn write(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("artifact has no parent")?;
    private_dir(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(data)
        .and_then(|()| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
pub(super) fn read(path: &Path) -> Result<String, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(super::process::OUTPUT_LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > super::process::OUTPUT_LIMIT {
        return Err("artifact exceeds 20 MB; split the collector scope".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn globs(patterns: &[String]) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern).map_err(|e| e.to_string())?);
    }
    builder.build().map_err(|e| e.to_string())
}
fn paths(
    root: &Path,
    patterns: &[String],
    generated: bool,
    excluded: Option<&Path>,
) -> Result<Vec<PathBuf>, String> {
    let glob = globs(patterns)?;
    let mut found = BTreeSet::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if excluded.is_some_and(|path| e.path().starts_with(path)) {
                return false;
            }
            let name = e.file_name().to_string_lossy();
            !e.file_type().is_dir()
                || !(matches!(
                    name.as_ref(),
                    ".git"
                        | ".scryr"
                        | ".cache"
                        | ".venv"
                        | "node_modules"
                        | "__pycache__"
                        | ".next"
                ) || (!generated
                    && matches!(name.as_ref(), "target" | "dist" | "build" | "coverage")))
        })
    {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry.path().strip_prefix(root).map_err(|e| e.to_string())?;
        if (patterns.is_empty()
            && !matches!(
                entry.path().extension().and_then(|s| s.to_str()),
                Some("scry")
            ))
            || glob.is_match(relative)
        {
            found.insert(entry.path().to_owned());
        }
        if found.len() > 50_000 {
            return Err(
                "collector input exceeds 50000 files; narrow its directory/watch patterns".into(),
            );
        }
    }
    Ok(found.into_iter().collect())
}
pub(super) fn reports(
    root: &Path,
    source_directory: &Path,
    patterns: &[String],
) -> Result<Vec<String>, String> {
    let base = super::process::directory(root, source_directory, ".")?;
    let files = paths(&base, patterns, true, None)?;
    if files.len() > 100 {
        return Err("report input exceeds 100 files".into());
    }
    let mut total = 0;
    files
        .iter()
        .map(|p| {
            let data = read(p)?;
            total += data.len();
            if total > 20_000_000 {
                return Err("combined report artifacts exceed 20 MB".into());
            }
            Ok(data)
        })
        .collect()
}
#[cfg(test)]
pub(super) fn input(root: &Path, config: &CollectorConfig) -> Result<String, String> {
    input_ignoring(root, root, config, None)
}

pub(super) fn input_ignoring(
    root: &Path,
    source_directory: &Path,
    config: &CollectorConfig,
    excluded: Option<&Path>,
) -> Result<String, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let root = root.as_path();
    let base = super::process::directory(root, source_directory, ".")?;
    let source_scope = base.strip_prefix(root).map_err(|e| e.to_string())?;
    if config.schedule().watch.is_empty()
        && matches!(
            config,
            C::Openmetrics(_) | C::DockerStats(_) | C::GithubPullRequests(_) | C::GithubActions(_)
        )
    {
        return fingerprint(&(source_scope, config.kind()));
    }
    let patterns = match config {
        C::Junit(c) => &c.files,
        C::Lcov(c) => &c.files,
        C::Cobertura(c) => &c.files,
        _ => &config.schedule().watch,
    };
    let directory = super::process::directory(root, &base, config.directory())?;
    let files = paths(&directory, patterns, !patterns.is_empty(), excluded)?;
    let mut hash = Sha256::new();
    hash.update(source_scope.to_string_lossy().as_bytes());
    hash.update([0]);
    let mut total = 0u64;
    for path in files {
        hash.update(
            path.strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .as_bytes(),
        );
        hash.update([0]);
        let mut file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        let mut buffer = vec![0; 64 * 1024];
        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > 100_000_000 {
                return Err(
                    "collector input exceeds 100 MB; narrow its directory/watch patterns".into(),
                );
            }
            hash.update(&buffer[..n]);
        }
        hash.update([0]);
    }
    Ok(hash
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        }))
}
pub(super) fn prune_runs(state: &Path, keep: &BTreeSet<PathBuf>) -> Result<(), String> {
    let runs = state.join("runs");
    if !runs.exists() {
        return Ok(());
    }
    let now = std::time::SystemTime::now();
    let mut directories = Vec::new();
    let mut total = 0;
    for entry in std::fs::read_dir(runs).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let size = walkdir::WalkDir::new(entry.path())
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|f| f.file_type().is_file())
            .filter_map(|f| f.metadata().ok())
            .map(|m| m.len())
            .sum::<u64>();
        total += size;
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .map_err(|e| e.to_string())?;
        directories.push((modified, entry.path(), size));
    }
    directories.sort();
    for (modified, path, size) in directories {
        if keep.iter().any(|p| p.starts_with(&path)) {
            continue;
        }
        if total > 500_000_000 || now.duration_since(modified).unwrap_or_default().as_secs() > 86400
        {
            std::fs::remove_dir_all(path).map_err(|e| e.to_string())?;
            total = total.saturating_sub(size);
        }
    }
    if total > 500_000_000 {
        return Err(
            "active collector artifacts exceed the 500 MB quota; narrow inventory scope".into(),
        );
    }
    Ok(())
}
/// Preserve the producer's artifact time independently of this import attempt.
pub(super) fn source_time(
    root: &Path,
    source_directory: &Path,
    config: &CollectorConfig,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
    let patterns = match config {
        C::Junit(c) => &c.files,
        C::Lcov(c) => &c.files,
        C::Cobertura(c) => &c.files,
        _ => return Ok(None),
    };
    let base = super::process::directory(root, source_directory, ".")?;
    let mut oldest = None;
    for path in paths(&base, patterns, true, None)? {
        let time = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map_err(|e| e.to_string())?;
        oldest = Some(oldest.map_or(time, |prior: std::time::SystemTime| prior.min(time)));
    }
    Ok(oldest.map(chrono::DateTime::from))
}

/// Identical local input scopes share one content scan per scheduler tick.
pub(super) fn input_scope(config: &CollectorConfig) -> Result<String, String> {
    let patterns = match config {
        C::Junit(c) => &c.files,
        C::Lcov(c) => &c.files,
        C::Cobertura(c) => &c.files,
        _ => &config.schedule().watch,
    };
    let remote = matches!(
        config,
        C::Openmetrics(_) | C::DockerStats(_) | C::GithubPullRequests(_) | C::GithubActions(_)
    );
    fingerprint(&(
        config.directory(),
        patterns,
        remote.then_some(config.kind()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_entrypoint_inputs_ignore_root_files_and_include_the_source_scope()
    -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path().canonicalize().map_err(|e| e.to_string())?;
        let source = root.join("services/api");
        let sibling = root.join("services/worker");
        write(&root.join("input.py"), b"root input")?;
        write(&source.join("input.py"), b"nested input")?;
        write(&sibling.join("input.py"), b"nested input")?;
        let config: CollectorConfig = serde_json::from_value(
            serde_json::json!({"kind":"pytest","schedule":{"watch":["input.py"]}}),
        )
        .map_err(|e| e.to_string())?;
        let before = input_ignoring(&root, &source, &config, None)?;
        write(&root.join("input.py"), b"unrelated root edit")?;
        assert_eq!(before, input_ignoring(&root, &source, &config, None)?);
        assert_ne!(before, input_ignoring(&root, &sibling, &config, None)?);
        write(&source.join("input.py"), b"nested edit")?;
        assert_ne!(before, input_ignoring(&root, &source, &config, None)?);
        let remote: CollectorConfig = serde_json::from_value(
            serde_json::json!({"kind":"github_actions","repository":"example/api"}),
        )
        .map_err(|e| e.to_string())?;
        assert_ne!(
            input_ignoring(&root, &source, &remote, None)?,
            input_ignoring(&root, &sibling, &remote, None)?
        );
        let outside = tempfile::tempdir().map_err(|e| e.to_string())?;
        assert!(input_ignoring(&root, outside.path(), &config, None).is_err());
        Ok(())
    }
    #[test]
    fn shared_input_scopes_ignore_custom_collector_state() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path().canonicalize().map_err(|e| e.to_string())?;
        let state = root.join("custom-state");
        write(&root.join("source.py"), b"print('hello')")?;
        let ruff: CollectorConfig = serde_json::from_value(serde_json::json!({"kind":"ruff"}))
            .map_err(|e| e.to_string())?;
        let pytest: CollectorConfig = serde_json::from_value(serde_json::json!({"kind":"pytest"}))
            .map_err(|e| e.to_string())?;
        assert_eq!(input_scope(&ruff)?, input_scope(&pytest)?);
        let before = input_ignoring(&root, &root, &ruff, Some(&state))?;
        write(&state.join("collection/run.json"), b"generated result")?;
        assert_eq!(before, input_ignoring(&root, &root, &pytest, Some(&state))?);
        assert_ne!(before, input(&root, &ruff)?);
        write(&root.join("source.py"), b"print('changed')")?;
        assert_ne!(before, input_ignoring(&root, &root, &ruff, Some(&state))?);
        Ok(())
    }
    #[test]
    fn artifacts_and_hashes_are_bounded_and_deterministic() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path();
        write(&root.join("test.xml"), b"first")?;
        let config: CollectorConfig =
            serde_json::from_value(serde_json::json!({"kind":"junit","files":["*.xml"]}))
                .map_err(|e| e.to_string())?;
        let a = input(root, &config)?;
        write(&root.join("ignored.txt"), b"unrelated")?;
        assert_eq!(a, input(root, &config)?);
        write(&root.join("test.xml"), b"second")?;
        assert_ne!(a, input(root, &config)?);
        assert_eq!(reports(root, root, &["*.xml".into()])?, vec!["second"]);
        Ok(())
    }
}
