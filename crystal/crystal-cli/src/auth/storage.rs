//! Auth state file storage.

use super::StoredAuth;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Auth file override environment variable.
const AUTH_FILE_ENV: &str = "SCRYR_CLI_AUTH_FILE";

/// Remove one auth file if it exists.
pub(super) fn remove_auth_file(auth_file: &Path) -> Result<(), String> {
    if !auth_file.exists() {
        println!("No stored scryr auth state was found.");
        return Ok(());
    }

    fs::remove_file(auth_file)
        .map_err(|error| format!("Failed to remove {}: {error}", auth_file.display()))?;
    println!("Removed stored scryr auth state.");
    Ok(())
}

/// Read stored auth state from disk.
pub(super) fn read_auth_from_file(auth_file: &Path) -> Result<StoredAuth, String> {
    let body = fs::read_to_string(auth_file)
        .map_err(|error| format!("Failed to read {}: {error}", auth_file.display()))?;

    serde_json::from_str(&body)
        .map_err(|error| format!("Failed to parse {}: {error}", auth_file.display()))
}

/// Persist stored auth state to disk.
pub(super) fn persist_auth_to_file(auth_file: &Path, auth: &StoredAuth) -> Result<(), String> {
    if let Some(parent) = auth_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }

    let serialized = serde_json::to_string_pretty(auth)
        .map_err(|error| format!("Failed to serialize scryr-cli auth state: {error}"))?;
    fs::write(auth_file, serialized)
        .map_err(|error| format!("Failed to write {}: {error}", auth_file.display()))?;
    tighten_file_permissions(auth_file)?;
    Ok(())
}

/// Resolve the auth state path from environment.
pub(super) fn auth_file_path() -> Result<PathBuf, String> {
    auth_file_path_from_values(
        env::var(AUTH_FILE_ENV).ok(),
        env::var("HOME")
            .ok()
            .or_else(|| env::var("USERPROFILE").ok()),
    )
}

/// Resolve the auth state path from explicit values.
fn auth_file_path_from_values(
    auth_file_override: Option<String>,
    home_dir: Option<String>,
) -> Result<PathBuf, String> {
    if let Some(path) = auth_file_override {
        return Ok(PathBuf::from(path));
    }

    let home = home_dir.ok_or_else(|| {
        format!("HOME is not set. Set {AUTH_FILE_ENV} to choose a scryr auth file.")
    })?;

    Ok(Path::new(&home)
        .join(".config")
        .join("scryr")
        .join("scryr-cli-auth.json"))
}

#[cfg(unix)]
fn tighten_file_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("Failed to secure {} permissions: {error}", path.display()))
}

#[cfg(not(unix))]
fn tighten_file_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{auth_file_path_from_values, remove_auth_file};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    #[test]
    fn remove_auth_file_deletes_existing_auth_state() -> TestResult {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let auth_path = std::env::temp_dir().join(format!("scryr-cli-auth-{unique}.json"));
        let resolved_auth_path =
            auth_file_path_from_values(Some(auth_path.to_string_lossy().into_owned()), None)
                .map_err(std::io::Error::other)?;

        fs::write(&resolved_auth_path, "{}")?;
        assert_eq!(resolved_auth_path, auth_path);

        remove_auth_file(&resolved_auth_path).map_err(std::io::Error::other)?;

        assert!(
            !resolved_auth_path.exists(),
            "expected auth file to be removed"
        );
        Ok(())
    }

    #[test]
    fn remove_auth_file_is_a_no_op_for_missing_files() -> TestResult {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let missing_path =
            std::env::temp_dir().join(format!("scryr-cli-auth-missing-{unique}.json"));

        remove_auth_file(&missing_path).map_err(std::io::Error::other)?;
        assert!(!missing_path.exists(), "expected path to remain absent");
        Ok(())
    }

    #[test]
    fn auth_file_path_prefers_override_and_falls_back_to_home() -> TestResult {
        let override_path = auth_file_path_from_values(Some("/tmp/auth.json".to_string()), None)
            .map_err(std::io::Error::other)?;
        let default_path = auth_file_path_from_values(None, Some("/home/tester".to_string()))
            .map_err(std::io::Error::other)?;

        assert_eq!(override_path, PathBuf::from("/tmp/auth.json"));
        assert_eq!(
            default_path,
            PathBuf::from("/home/tester/.config/scryr/scryr-cli-auth.json")
        );
        Ok(())
    }
}
