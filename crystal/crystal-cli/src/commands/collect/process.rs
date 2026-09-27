//! Bounded, cancellable argv execution for trusted local integrations.
#![allow(clippy::missing_docs_in_private_items)]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;

pub(super) const OUTPUT_LIMIT: usize = 20_000_000;

#[derive(Debug)]
pub(super) struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: f64,
}

/// A separate process group guarantees cancellation also reaches tool children.
struct ProcessGroup(i32);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            let _ = nix::sys::signal::killpg(
                nix::unistd::Pid::from_raw(self.0),
                nix::sys::signal::Signal::SIGKILL,
            );
        }
    }
}

pub(super) fn resolve(executable: &str, directory: &Path) -> Result<PathBuf, String> {
    if executable.contains('/') {
        let path = directory.join(executable);
        return path
            .canonicalize()
            .map_err(|_| format!("missing tool: {executable}"));
    }
    for candidate in [
        directory.join(".venv/bin").join(executable),
        directory.join("node_modules/.bin").join(executable),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    for path in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let candidate = path.join(executable);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "missing tool: {executable}; install it in the project environment or PATH"
    ))
}

pub(super) fn directory(root: &Path, base: &Path, path: &str) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let path = base
        .join(path)
        .canonicalize()
        .map_err(|e| format!("working directory: {e}"))?;
    if !path.starts_with(&root) || !path.is_dir() {
        return Err("working directory must remain within the registered project".into());
    }
    Ok(path)
}

pub(super) async fn run(
    executable: &Path,
    args: &[String],
    cwd: &Path,
    env: &BTreeMap<String, String>,
    timeout: Duration,
) -> Result<Output, String> {
    let mut command = tokio::process::Command::new(executable);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .env_clear();
    for name in [
        "PATH",
        "HOME",
        "USER",
        "TMPDIR",
        "TMP",
        "TEMP",
        "LANG",
        "LC_ALL",
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "CARGO_HOME",
        "RUSTUP_HOME",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command.env("NO_COLOR", "1").env("CI", "1").envs(env);
    #[cfg(unix)]
    command.process_group(0);
    let start = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|e| format!("cannot start {}: {e}", executable.display()))?;
    let pid = child
        .id()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or("child process has no PID")?;
    let _group = ProcessGroup(pid);
    let stdout = child.stdout.take().ok_or("missing child stdout")?;
    let stderr = child.stderr.take().ok_or("missing child stderr")?;
    let task = async {
        let (stdout, stderr, status) =
            tokio::try_join!(read(stdout, OUTPUT_LIMIT), read(stderr, 128_000), async {
                child.wait().await.map_err(|e| e.to_string())
            },)?;
        Ok::<_, String>(Output {
            code: status.code().unwrap_or(-1),
            stdout: String::from_utf8(stdout).map_err(|_| "tool output is not UTF-8")?,
            stderr: redact(&String::from_utf8_lossy(&stderr), env),
            duration: start.elapsed().as_secs_f64(),
        })
    };
    tokio::time::timeout(timeout, task)
        .await
        .map_err(|_| format!("collector timed out after {}s", timeout.as_secs()))?
}

async fn read(reader: impl tokio::io::AsyncRead + Unpin, limit: usize) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut data)
        .await
        .map_err(|e| e.to_string())?;
    if data.len() > limit {
        return Err(format!("tool output exceeds {limit} bytes"));
    }
    Ok(data)
}

pub(super) fn redact(text: &str, env: &BTreeMap<String, String>) -> String {
    env.values()
        .filter(|value| value.len() >= 4)
        .fold(text.to_owned(), |out, value| {
            out.replace(value, "[redacted]")
        })
}

pub(super) fn success(output: &Output, codes: &[i32]) -> Result<(), String> {
    if codes.contains(&output.code) {
        return Ok(());
    }
    let message: String = output.stderr.chars().take(1500).collect();
    Err(format!(
        "tool exited with status {}: {message}",
        output.code
    ))
}

pub(super) async fn version(
    executable: &Path,
    cwd: &Path,
    env: &BTreeMap<String, String>,
) -> Result<String, String> {
    let output = run(
        executable,
        &["--version".into()],
        cwd,
        env,
        Duration::from_secs(10),
    )
    .await?;
    success(&output, &[0])?;
    Ok(output
        .stdout
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(256)
        .collect())
}

pub(super) fn check_version(actual: &str, requirement: &str) -> Result<(), String> {
    let required = semver::VersionReq::parse(requirement)
        .map_err(|e| format!("invalid tool version constraint: {e}"))?;
    let version = actual
        .split_whitespace()
        .find_map(|v| {
            semver::Version::parse(v.trim_end_matches([',', ';']).trim_start_matches('v')).ok()
        })
        .ok_or_else(|| format!("cannot determine tool version from {actual}"))?;
    if !required.matches(&version) {
        return Err(format!(
            "incompatible tool: {actual}; requires {requirement}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn nonzero_output_and_timeout_are_distinct() -> Result<(), String> {
        let out = run(
            Path::new("/bin/sh"),
            &["-c".into(), "printf result; exit 1".into()],
            Path::new("/tmp"),
            &BTreeMap::new(),
            Duration::from_secs(2),
        )
        .await?;
        assert_eq!(out.stdout, "result");
        success(&out, &[0, 1])?;
        assert!(success(&out, &[0]).is_err());
        let error = run(
            Path::new("/bin/sh"),
            &["-c".into(), "sleep 30 & wait".into()],
            Path::new("/tmp"),
            &BTreeMap::new(),
            Duration::from_millis(20),
        )
        .await
        .err()
        .ok_or("expected timeout")?;
        assert!(error.contains("timed out"));
        Ok(())
    }
    #[test]
    fn versions_and_redaction() -> Result<(), String> {
        check_version("syft 1.42.3", ">=1,<2")?;
        assert!(check_version("syft 2.0.0", ">=1,<2").is_err());
        assert_eq!(
            redact(
                "token=secret-value",
                &BTreeMap::from([("TOKEN".into(), "secret-value".into())])
            ),
            "token=[redacted]"
        );
        Ok(())
    }
}
