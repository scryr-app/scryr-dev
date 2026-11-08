//! Bounded GitHub CLI transport. Provider values are arguments, never shell code.
#![allow(clippy::missing_docs_in_private_items)]
use serde_json::Value;
use std::io::{Read, Seek};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const MAX_OUTPUT: u64 = 20_000_000;

pub(super) struct GitHub {
    pub host: String,
    #[cfg(test)]
    responses: Option<std::collections::HashMap<String, Value>>,
}

impl GitHub {
    pub(super) const fn new(host: String) -> Self {
        Self {
            host,
            #[cfg(test)]
            responses: None,
        }
    }
    #[cfg(test)]
    pub(super) fn fixture(responses: std::collections::HashMap<String, Value>) -> Self {
        Self {
            host: "github.com".into(),
            responses: Some(responses),
        }
    }
    pub(super) fn json(&self, endpoint: &str) -> Result<Value, String> {
        #[cfg(test)]
        if let Some(responses) = &self.responses {
            return responses
                .get(endpoint)
                .cloned()
                .ok_or_else(|| format!("Unconfigured fixture request: {endpoint}"));
        }
        let bytes = execute(&["api", "--hostname", &self.host, "--method", "GET", endpoint])?;
        serde_json::from_slice(&bytes).map_err(|_| "GitHub returned malformed JSON".into())
    }

    /// Follow page numbers explicitly, retaining a bounded recent history.
    pub(super) fn list(
        &self,
        endpoint: &str,
        field: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Value>, String> {
        let mut result = Vec::new();
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        let page_size = limit.min(100);
        for page in 1..=100 {
            let response = self.json(&format!(
                "{endpoint}{separator}per_page={page_size}&page={page}"
            ))?;
            let value = field.map_or(&response, |key| &response[key]);
            let batch = value
                .as_array()
                .ok_or("GitHub list response is not an array")?;
            result.extend(batch.iter().cloned());
            if result.len() >= limit as usize || batch.len() < page_size as usize {
                result.truncate(limit as usize);
                return Ok(result);
            }
        }
        Err("GitHub pagination limit exceeded".into())
    }
}

pub(super) fn execute(args: &[&str]) -> Result<Vec<u8>, String> {
    execute_bounded(args, MAX_OUTPUT, None)
}

pub(super) fn execute_bounded(
    args: &[&str],
    maximum: u64,
    directory: Option<&std::path::Path>,
) -> Result<Vec<u8>, String> {
    let mut stdout = tempfile::tempfile().map_err(|e| e.to_string())?;
    let stderr = tempfile::tempfile().map_err(|e| e.to_string())?;
    let mut child = Command::new("gh")
        .args(args)
        .env("GH_PROMPT_DISABLED", "1")
        .env("GH_PAGER", "cat")
        .stdin(Stdio::null())
        .stdout(stdout.try_clone().map_err(|e| e.to_string())?)
        .stderr(stderr.try_clone().map_err(|e| e.to_string())?)
        .spawn()
        .map_err(
            |_| "Unable to start gh; install GitHub CLI and authenticate with gh auth login",
        )?;
    let started = Instant::now();
    loop {
        if started.elapsed() > Duration::from_secs(45)
            || stdout.metadata().map_err(|e| e.to_string())?.len() > maximum
            || stderr.metadata().map_err(|e| e.to_string())?.len() > MAX_OUTPUT
            || directory
                .is_some_and(|path| directory_size(path).map_or(true, |size| size > maximum))
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err("GitHub request exceeded its time or output limit".into());
        }
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            if !status.success() {
                // Do not echo provider errors: they can contain credentials or private URLs.
                return Err(format!(
                    "GitHub request failed ({status}); check gh authentication and repository read permissions"
                ));
            }
            stdout.rewind().map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            stdout
                .take(maximum + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() as u64 > maximum {
                return Err("GitHub response exceeds its size limit".into());
            }
            return Ok(bytes);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn directory_size(path: &std::path::Path) -> Result<u64, std::io::Error> {
    let mut size = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.path().symlink_metadata()?;
        if metadata.is_symlink() {
            return Err(std::io::Error::other("Symlink in report artifact"));
        }
        size += if metadata.is_dir() {
            directory_size(&entry.path())?
        } else {
            metadata.len()
        };
    }
    Ok(size)
}

pub(super) fn segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn endpoint_segments_cannot_inject_query_parameters() {
        assert_eq!(
            super::segment(".github/workflows/build.yml"),
            ".github%2Fworkflows%2Fbuild.yml"
        );
        assert_eq!(super::segment("prod&per_page=100"), "prod%26per_page%3D100");
    }
}
