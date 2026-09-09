//! End-to-end native reporter HTTP contract tests.
use std::io::{Read, Write};

#[test]
fn reporter_prefers_main_then_master_and_posts_to_crystal() -> Result<(), Box<dyn std::error::Error>>
{
    for (main_exists, selected) in [(true, "main"), (false, "master")] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let selected = selected.to_owned();
        let event = tempfile::NamedTempFile::new()?;
        std::fs::write(event.path(), serde_json::json!({
            "repository": {"id": 123, "full_name": "example/api", "default_branch": "develop"},
            "workflow_run": {"id": 100, "workflow_id": 42, "name": "CI", "run_attempt": 1,
                "head_branch": selected, "head_sha": "abc", "html_url": "https://github.com/example/api/actions/runs/100",
                "status": "completed", "conclusion": "failure", "created_at": "2026-09-08T10:00:00Z", "updated_at": "2026-09-08T10:02:00Z"}
        }).to_string())?;
        let server = std::thread::spawn(move || -> Result<(), String> {
            let paths = if main_exists {
                vec!["/repos/example/api/branches/main", "/graphql"]
            } else {
                vec![
                    "/repos/example/api/branches/main",
                    "/repos/example/api/branches/master",
                    "/graphql",
                ]
            };
            for path in paths {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && std::time::Instant::now() < deadline =>
                        {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                        Err(error) => return Err(error.to_string()),
                    }
                };
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .map_err(|e| e.to_string())?;
                let mut bytes = vec![0; 16384];
                let n = stream.read(&mut bytes).map_err(|e| e.to_string())?;
                let request = String::from_utf8_lossy(&bytes[..n]);
                if !request
                    .lines()
                    .next()
                    .is_some_and(|line| line.contains(path))
                {
                    return Err(format!("unexpected request for {path}"));
                }
                if path == "/graphql"
                    && (!request.contains("recordActionRun") || !request.contains("services/api"))
                {
                    return Err("missing mutation payload".into());
                }
                let status = if path.ends_with("/main") && !main_exists {
                    "404 Not Found"
                } else {
                    "200 OK"
                };
                let body = if path == "/graphql" {
                    r#"{"data":{"recordActionRun":true}}"#
                } else {
                    "{}"
                };
                write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).map_err(|e| e.to_string())?;
            }
            Ok(())
        });
        let mut command = assert_cmd::cargo::cargo_bin_cmd!("scryr");
        command
            .args([
                "report-action-status",
                "--manifest-id",
                "services/api",
                "--event-file",
            ])
            .arg(event.path())
            .arg("--endpoint")
            .arg(format!("http://{address}/graphql"))
            .env("GITHUB_TOKEN", "test-token")
            .env("GITHUB_API_URL", format!("http://{address}"))
            .env_remove("SCRYR_TOKEN")
            .env_remove("SCRYR_CLERK_ORG_ID");
        command
            .assert()
            .success()
            .stdout(predicates::str::contains("Recorded action status"));
        server.join().map_err(|_| "mock server failed")??;
    }
    Ok(())
}
