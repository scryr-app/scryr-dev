//! Managed `uv` path helpers.

use std::path::{Path, PathBuf};

/// Managed `uv` executable installed into Scryr local state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedUv {
    /// Path to the managed `uv` executable.
    pub(super) executable: PathBuf,
}

impl ManagedUv {
    /// Path to the managed `uv` executable.
    pub(crate) fn executable(&self) -> &Path {
        &self.executable
    }
}

/// Return the managed `uv` executable path.
pub(super) fn managed_uv_path(scryr_dir: &Path) -> PathBuf {
    scryr_dir.join("bin").join(executable_name("uv"))
}

/// Return the executable filename for the current platform.
fn executable_name(name: &str) -> String {
    #[cfg(windows)]
    {
        if name.ends_with(".exe") {
            name.to_string()
        } else {
            format!("{name}.exe")
        }
    }

    #[cfg(not(windows))]
    {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::managed_uv_path;
    use std::path::Path;

    #[test]
    fn managed_uv_path_lives_under_scryr_bin() {
        assert!(managed_uv_path(Path::new("/tmp/.scryr")).ends_with(".scryr/bin/uv"));
    }
}
