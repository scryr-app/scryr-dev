//! Native CLI contracts, including complete and interrupted provider snapshots.
use std::io::{Read, Write};

#[test]
fn dependabot_requires_all_pages_before_emitting_a_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    for complete in [true, false] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let server = std::thread::spawn(move || -> Result<(), String> {
            for page in 1..=2 {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((s, _)) => break s,
                        Err(e)
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                && std::time::Instant::now() < deadline =>
                        {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
                        Err(e) => return Err(e.to_string()),
                    }
                };
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .map_err(|e| e.to_string())?;
                let mut bytes = [0; 8192];
                let n = stream.read(&mut bytes).map_err(|e| e.to_string())?;
                let request = String::from_utf8_lossy(&bytes[..n]);
                if !request.contains(&format!("page={page}")) {
                    return Err("incorrect pagination".into());
                }
                let body = if page == 1 {
                    r#"[{"number":1,"state":"open","dependency":{"manifest_path":"api/uv.lock","package":{"name":"example","ecosystem":"pip"}},"security_advisory":{"severity":"high"},"html_url":"https://github.com/example/api/security/dependabot/1"}]"#
                } else {
                    "[]"
                };
                let status = if page == 2 && !complete {
                    "403 Forbidden"
                } else {
                    "200 OK"
                };
                write!(stream,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).map_err(|e|e.to_string())?;
            }
            Ok(())
        });
        let mut command = assert_cmd::cargo::cargo_bin_cmd!("scryr");
        command
            .args([
                "report",
                "dependencies",
                "--manifest-id",
                "api",
                "--repository",
                "example/api",
                "--manifest-path",
                "api/uv.lock",
                "--run-id",
                "snapshot-1",
                "--observed-at",
                "2026-09-08T10:00:00Z",
                "--dry-run",
            ])
            .env("GITHUB_TOKEN", "fixture")
            .env("GITHUB_API_URL", format!("http://{address}"));
        if complete {
            command
                .assert()
                .success()
                .stdout(predicates::str::contains("\"severity\": \"high\""));
        } else {
            command
                .assert()
                .failure()
                .stdout("")
                .stderr(predicates::str::contains("no snapshot uploaded"));
        }
        server.join().map_err(|_| "mock server failed")??;
    }
    Ok(())
}
