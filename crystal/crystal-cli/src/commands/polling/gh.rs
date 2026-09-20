//! Read GitHub using gh's existing authentication; never extract its token.
use serde_json::Value;
use std::{path::PathBuf, time::Duration};

/// The subprocess boundary is injectable for offline regression tests.
pub(super) struct Gh {
    /// gh executable resolved through PATH in production.
    pub executable: PathBuf,
}

impl Default for Gh {
    fn default() -> Self {
        Self {
            executable: "gh".into(),
        }
    }
}

impl Gh {
    /// Execute a read-only API call, killing the child on timeout or cancellation.
    pub(super) async fn get(&self, path: &str) -> Result<Value, String> {
        self.request(path, false)
            .await?
            .ok_or_else(|| "gh returned an empty response".into())
    }

    /// Read every response page as a JSON array of pages.
    pub(super) async fn get_pages(&self, path: &str) -> Result<Value, String> {
        self.request(path, true)
            .await?
            .ok_or_else(|| "gh returned an empty response".into())
    }

    /// A successful empty response denotes a pending asynchronous SBOM export.
    pub(super) async fn get_pending(&self, path: &str) -> Result<Option<Value>, String> {
        self.request(path, false).await
    }

    /// Keep pagination, size, authentication and cancellation handling at one boundary.
    async fn request(&self, path: &str, paginate: bool) -> Result<Option<Value>, String> {
        let mut command = tokio::process::Command::new(&self.executable);
        command.args(["api", "--hostname", "github.com", "--method", "GET", path]);
        if paginate {
            command.args(["--paginate", "--slurp"]);
        }
        let output = command
            .env("GH_PROMPT_DISABLED", "1")
            .env("GH_PAGER", "cat")
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .output();
        let output = tokio::time::timeout(Duration::from_secs(30), output)
            .await
            .map_err(|_| "GitHub request timed out; collection will retry")?
            .map_err(|_| "Cannot run gh; install GitHub CLI and run `gh auth login`")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(if stderr.contains("rate limit") || stderr.contains("429") {
                "GitHub rate limit reached; collection will back off and retry"
            } else if stderr.contains("(HTTP 404)") {
                "GitHub resource unavailable (HTTP 404); check repository access and enabled features"
            } else {
                "GitHub request failed; check `gh auth status` and repository permissions"
            }
            .into());
        }
        if output.stdout.len() > 16 * 1024 * 1024 {
            return Err("GitHub response exceeded the collection size limit".into());
        }
        if output.stdout.iter().all(u8::is_ascii_whitespace) {
            return Ok(None);
        }
        serde_json::from_slice(&output.stdout)
            .map(Some)
            .map_err(|_| "gh returned invalid JSON".into())
    }
}
