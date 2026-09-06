use std::fs;
use std::path::{Path, PathBuf};

/// Directory name used for Scryr-managed local state.
const SCRYR_DIR_NAME: &str = ".scryr";

/// Resolve the Scryr state directory from an explicit override or the user's home directory.
pub(crate) fn resolve_scryr_dir(override_dir: Option<&Path>) -> Result<PathBuf, String> {
    let scryr_dir = match override_dir {
        Some(path) => path.to_path_buf(),
        None => default_scryr_dir()?,
    };

    fs::create_dir_all(&scryr_dir).map_err(|error| {
        format!(
            "Failed to create Scryr directory {}: {error}",
            scryr_dir.display()
        )
    })?;
    scryr_dir.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve Scryr directory {}: {error}",
            scryr_dir.display()
        )
    })
}

/// Return the default Scryr state directory in the current manifest project.
fn default_scryr_dir() -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir()
        .map_err(|error| format!("Failed to resolve current directory: {error}"))?;
    Ok(default_scryr_dir_from_current_dir(&current_dir))
}

/// Pure implementation for default directory resolution.
fn default_scryr_dir_from_current_dir(current_dir: &Path) -> PathBuf {
    current_dir.join(SCRYR_DIR_NAME)
}

#[cfg(test)]
mod tests {
    use super::default_scryr_dir_from_current_dir;
    use std::path::{Path, PathBuf};

    #[test]
    fn default_scryr_dir_lives_under_current_project() {
        let resolved = default_scryr_dir_from_current_dir(Path::new("/workspace/example"));

        assert_eq!(resolved, PathBuf::from("/workspace/example/.scryr"));
    }
}
