//! Remote path and shell quoting helpers for Sprite execution.

use std::path::Path;

/// Convert a relative local path to a portable remote path fragment.
pub(super) fn relative_path_text(path: &Path) -> Result<String, String> {
    if path.is_absolute() {
        return Err(format!("Expected a relative path, got {}", path.display()));
    }
    Ok(path.to_string_lossy().replace('\\', "/"))
}

/// Return the final path component as UTF-8 text.
pub(super) fn relative_file_name(path: &Path) -> Result<&str, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Path has no UTF-8 file name: {}", path.display()))
}

/// Join remote path fragments with one slash.
pub(super) fn remote_path_join(root: &str, relative: &str) -> String {
    format!(
        "{}/{}",
        root.trim_end_matches('/'),
        relative.trim_start_matches('/')
    )
}

/// Quote a value for POSIX shell single-quoted string context.
pub(super) fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{remote_path_join, shell_single_quote};

    #[test]
    fn shell_single_quote_escapes_embedded_quotes() {
        assert_eq!(shell_single_quote("a'b"), "'a'\\''b'");
    }

    #[test]
    fn remote_path_join_handles_extra_slashes() {
        assert_eq!(
            remote_path_join("/tmp/root/", "/file.py"),
            "/tmp/root/file.py"
        );
    }
}
