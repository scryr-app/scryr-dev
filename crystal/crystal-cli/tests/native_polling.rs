//! Offline end-to-end coverage of the installed CLI's provider lifecycle.
#![cfg(unix)]
#![allow(clippy::missing_docs_in_private_items)]

use serde_json::{Value, json};
use std::error::Error;
use std::fs;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

struct Fixture {
    directory: TempDir,
    port: u16,
    http: reqwest::blocking::Client,
}

impl Fixture {
    fn new() -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        fs::create_dir_all(root.join(".scryr/bin"))?;
        fs::create_dir_all(root.join("bin"))?;
        fs::write(
            root.join("index.scry"),
            "api = Manifest(name='API')\ndiagram = Diagram(name='System', manifests=[api])\n",
        )?;
        // Mock only Python execution; native envelope conversion, checks, upload,
        // database, HTTP, scheduling, gh execution, and reporting remain real.
        executable(
            &root.join(".scryr/bin/uv"),
            r#"#!/bin/sh
set -eu
case "$1" in
 --version) printf 'uv 0.12.7\n';;
 python|sync) exit 0;;
 run)
   flag=""
   for argument in "$@"; do flag="$argument"; done
   case "$flag" in
     --schema) printf '{"title":"Manifest"}\n';;
     --json) /bin/cat "$TEST_FIXTURE/manifest.json";;
     --format|--format-check|--lint|--typecheck) exit 0;;
     *) exit 2;;
   esac;;
 *) exit 2;;
esac
"#,
        )?;
        executable(
            &root.join("bin/gh"),
            r#"#!/bin/sh
set -eu
test "$1" = api
test "$2" = --hostname
test "$3" = github.com
test "$4" = --method
test "$5" = GET
printf '%s\n' "$6" >> "$TEST_FIXTURE/gh.calls"
if test -f "$TEST_FIXTURE/block"; then
  printf '%s\n' "$$" > "$TEST_FIXTURE/gh.pid"
  exec /bin/sleep 60
fi
if test -f "$TEST_FIXTURE/fail"; then
  printf 'HTTP 429 rate limit private-provider-detail\n' >&2
  exit 1
fi
case "$6" in
 repos/example/api) /bin/cat "$TEST_FIXTURE/repository.json";;
 repos/example/api/actions/workflows/ci.yml) /bin/cat "$TEST_FIXTURE/workflow.json";;
 repos/example/api/actions/workflows/ci.yml/runs\?*) /bin/cat "$TEST_FIXTURE/runs.json";;
 repos/example/api/actions/runs/*/attempts/1/jobs\?*) /bin/cat "$TEST_FIXTURE/jobs.json";;
 *) printf 'Unexpected endpoint %s\n' "$6" >&2; exit 2;;
esac
"#,
        )?;
        let manifest = json!({"name":"API", "manifestId":"services/api",
            "github":{"repoUrl":"https://github.com/example/api"},
            "cicd":{"platform":"github_actions","source":{"workflows":["ci.yml"],"branch":"main"}}});
        fs::write(root.join("manifest.json"), json!([
            {"variable_name":"api","manifest":manifest},
            {"kind":"diagram","variable_name":"diagram","diagram":{"name":"System","manifests":[manifest]}}
        ]).to_string())?;
        fs::write(
            root.join("repository.json"),
            json!({"id":123,"full_name":"example/api","default_branch":"main"}).to_string(),
        )?;
        fs::write(
            root.join("workflow.json"),
            json!({"id":42,"name":"CI","path":".github/workflows/ci.yml"}).to_string(),
        )?;
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let fixture = Self {
            directory,
            port,
            http: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?,
        };
        fixture.observation(false)?;
        Ok(fixture)
    }

    fn command(&self) -> Command {
        let root = self.directory.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_scryr"));
        // No ambient tokens, cloud DB settings, GitHub config, or executable paths.
        command
            .env_clear()
            .current_dir(root)
            .env("HOME", root)
            .env("PATH", root.join("bin"))
            .env("TEST_FIXTURE", root)
            .env("SCRYR_SQLITE_PATH", root.join("database.sqlite"))
            .env("AUTH_MODE", "local")
            .env("RUST_LOG", "warn");
        command
    }

    fn serve(&self, extra: &[&str]) -> TestResult<Server> {
        let log = fs::File::create(self.directory.path().join("server.log"))?;
        let mut command = self.command();
        if extra.contains(&"--server-only") {
            command.arg("serve");
        } else {
            command.arg("serve").arg("--no-open");
        }
        let child = command
            .args(["--host", "127.0.0.1", "--port", &self.port.to_string()])
            .args(extra)
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .spawn()?;
        let server = Server { child };
        self.wait_for(Duration::from_secs(15), || {
            self.http
                .get(format!("{}/ready", self.base()))
                .send()
                .is_ok_and(|r| r.status().is_success())
        })?;
        Ok(server)
    }

    fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    fn query(&self, query: &str) -> TestResult<Value> {
        let body: Value = self
            .http
            .post(format!("{}/graphql", self.base()))
            .json(&json!({"query":query}))
            .send()?
            .error_for_status()?
            .json()?;
        if body["errors"]
            .as_array()
            .is_some_and(|errors| !errors.is_empty())
        {
            return Err(format!("GraphQL rejected test query: {body}").into());
        }
        Ok(body["data"].clone())
    }

    fn history(&self) -> TestResult<Value> {
        Ok(
            self.query("{actionHistory(manifestId:\"services/api\",limit:100)}")?["actionHistory"]
                .clone(),
        )
    }

    fn block(&self) -> TestResult<Value> {
        let data = self.query("{blocks(scryIdentifier:\"diagram\"){rawJsonString}}")?;
        let raw = data["blocks"][0]["rawJsonString"]
            .as_str()
            .ok_or("Diagram not loaded")?;
        Ok(serde_json::from_str(raw)?)
    }

    fn wait_for(&self, timeout: Duration, condition: impl Fn() -> bool) -> TestResult {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if condition() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err(format!(
            "Timed out after {timeout:?}; server log:\n{}",
            fs::read_to_string(self.directory.path().join("server.log")).unwrap_or_default()
        )
        .into())
    }

    fn sync(&self) -> TestResult<Output> {
        Ok(self
            .command()
            .args([
                "sync",
                "github",
                "--manifest",
                "api",
                "--endpoint",
                &format!("{}/graphql", self.base()),
                "--json",
            ])
            .output()?)
    }

    fn observation(&self, completed: bool) -> TestResult {
        let status = if completed {
            "completed"
        } else {
            "in_progress"
        };
        let conclusion = completed.then_some("success");
        let updated = if completed {
            "2026-09-08T10:02:00Z"
        } else {
            "2026-09-08T10:00:00Z"
        };
        let root = self.directory.path();
        // Rename prevents a concurrent collector from reading a partial fixture.
        write_json(
            root,
            "runs.json",
            &json!({"workflow_runs":[{
                "id":100,"workflow_id":42,"name":"CI","path":".github/workflows/ci.yml","run_attempt":1,
                "head_branch":"main","head_sha":"abcdef","html_url":"https://github.com/example/api/actions/runs/100",
                "status":status,"conclusion":conclusion,"created_at":"2026-09-08T10:00:00Z","updated_at":updated,
                "repository":{"id":123,"full_name":"example/api"}
            }]}),
        )?;
        write_json(
            root,
            "jobs.json",
            &json!({"total_count":1,"jobs":[{
                "id":1,"name":"Tests","run_id":100,"run_attempt":1,"status":status,"conclusion":conclusion,
                "started_at":"2026-09-08T10:00:00Z","completed_at":completed.then_some(updated),
                "html_url":"https://github.com/example/api/actions/runs/100/jobs/1"
            }]}),
        )
    }

    fn calls(&self) -> usize {
        fs::read_to_string(self.directory.path().join("gh.calls"))
            .unwrap_or_default()
            .lines()
            .count()
    }
}

struct Server {
    child: Child,
}
impl Server {
    fn stop(&mut self) -> TestResult {
        if self.child.try_wait()?.is_some() {
            return Ok(());
        }
        Command::new("/bin/kill")
            .args(["-INT", &self.child.id().to_string()])
            .status()?;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(8) {
            if self.child.try_wait()?.is_some() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(50));
        }
        self.child.kill()?;
        self.child.wait()?;
        Err("Server did not shut down after SIGINT".into())
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        if self.stop().is_err() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn executable(path: &Path, body: &str) -> TestResult {
    fs::write(path, body)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn write_json(root: &Path, name: &str, value: &Value) -> TestResult {
    let staged = root.join(format!("{name}.tmp"));
    fs::write(&staged, value.to_string())?;
    fs::rename(staged, root.join(name))?;
    Ok(())
}

#[test]
fn serve_polls_immediately_and_refreshes_without_watch() -> TestResult {
    let fixture = Fixture::new()?;
    let mut server = fixture.serve(&["--poll", "15"])?;
    fixture.wait_for(Duration::from_secs(10), || {
        fixture
            .history()
            .is_ok_and(|h| h["runs"][0]["status"] == "in_progress")
    })?;
    let initial = fixture.calls();
    fixture.observation(true)?;
    fixture.wait_for(Duration::from_secs(22), || {
        fixture
            .history()
            .is_ok_and(|h| h["runs"][0]["status"] == "completed")
    })?;
    let history = fixture.history()?;
    assert_eq!(history["runs"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        history["runs"][0]["events"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(history["runs"][0]["jobs"][0]["conclusion"], "success");
    assert!(fixture.calls() > initial);
    assert_eq!(fixture.block()?["cicd"]["buildStatus"], "passing");
    server.stop()?;
    assert!(
        fixture
            .http
            .get(format!("{}/ready", fixture.base()))
            .send()
            .is_err()
    );
    Ok(())
}

#[test]
fn one_shot_sync_deduplicates_preserves_errors_and_recovers() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.observation(true)?;
    let mut server = fixture.serve(&["--no-poll"])?;
    fixture.wait_for(Duration::from_secs(10), || fixture.block().is_ok())?;
    thread::sleep(Duration::from_millis(300));
    assert_eq!(fixture.calls(), 0);
    let first = fixture.sync()?;
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let summary: Value = serde_json::from_slice(&first.stdout)?;
    assert_eq!(summary["recorded"], 1);
    let second = fixture.sync()?;
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&second.stdout)?["recorded"],
        0
    );
    let original = fixture.history()?;
    fs::write(fixture.directory.path().join("fail"), "")?;
    let failed = fixture.sync()?;
    assert!(!failed.status.success());
    assert!(!String::from_utf8_lossy(&failed.stderr).contains("private-provider-detail"));
    assert_eq!(fixture.history()?, original);
    let block = fixture.block()?;
    assert_eq!(block["cicd"]["buildStatus"], "passing");
    assert!(block["providerSync"]["github"]["error"].is_string());
    fs::remove_file(fixture.directory.path().join("fail"))?;
    assert!(fixture.sync()?.status.success());
    assert_eq!(fixture.history()?, original);
    assert!(fixture.block()?["providerSync"]["github"]["error"].is_null());
    server.stop()?;
    let _restarted = fixture.serve(&["--no-poll"])?;
    let repeated = fixture.sync()?;
    assert!(repeated.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&repeated.stdout)?["recorded"],
        0
    );
    assert_eq!(fixture.history()?, original);
    Ok(())
}

#[test]
fn missing_gh_keeps_the_loaded_diagram_usable() -> TestResult {
    let fixture = Fixture::new()?;
    fs::remove_file(fixture.directory.path().join("bin/gh"))?;
    let _server = fixture.serve(&["--poll", "15"])?;
    fixture.wait_for(Duration::from_secs(10), || {
        fixture
            .block()
            .is_ok_and(|b| b["providerSync"]["github"]["error"].is_string())
    })?;
    let block = fixture.block()?;
    assert_eq!(block["name"], "API");
    assert!(
        block["providerSync"]["github"]["error"]
            .as_str()
            .is_some_and(|e| e.contains("gh auth login"))
    );
    assert_eq!(fixture.history()?["runs"].as_array().map(Vec::len), Some(0));
    assert_eq!(fixture.calls(), 0);
    Ok(())
}

#[test]
fn server_only_and_unconfigured_projects_do_not_call_gh() -> TestResult {
    let fixture = Fixture::new()?;
    let mut server = fixture.serve(&["--server-only"])?;
    // Push a real declaration to prove server-only never polls persisted blocks.
    let pushed = fixture
        .command()
        .args(["push", "--endpoint", &format!("{}/graphql", fixture.base())])
        .output()?;
    assert!(
        pushed.status.success(),
        "{}",
        String::from_utf8_lossy(&pushed.stderr)
    );
    assert!(fixture.block().is_ok());
    thread::sleep(Duration::from_millis(300));
    assert_eq!(fixture.calls(), 0);
    server.stop()?;
    let root = fixture.directory.path();
    let mut values: Value = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)?;
    values[0]["manifest"]["cicd"]["source"] = Value::Null;
    values[1]["diagram"]["manifests"][0]["cicd"]["source"] = Value::Null;
    write_json(root, "manifest.json", &values)?;
    let _server = fixture.serve(&["--poll", "15"])?;
    fixture.wait_for(Duration::from_secs(10), || {
        fixture.block().is_ok_and(|b| b["cicd"]["source"].is_null())
    })?;
    thread::sleep(Duration::from_millis(300));
    assert_eq!(fixture.calls(), 0);
    Ok(())
}

#[test]
fn serve_shutdown_cancels_an_in_flight_gh_process() -> TestResult {
    let fixture = Fixture::new()?;
    fs::write(fixture.directory.path().join("block"), "")?;
    let mut server = fixture.serve(&["--poll", "15"])?;
    let pid_file = fixture.directory.path().join("gh.pid");
    fixture.wait_for(Duration::from_secs(10), || pid_file.is_file())?;
    let pid = fs::read_to_string(pid_file)?;
    server.stop()?;
    fixture.wait_for(Duration::from_secs(3), || {
        Command::new("/bin/kill")
            .args(["-0", pid.trim()])
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| !s.success())
    })?;
    Ok(())
}

#[test]
fn watched_source_changes_refresh_subscriptions_without_waiting_for_the_interval() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.observation(true)?;
    let _server = fixture.serve(&["--watch", "--poll", "3600"])?;
    fixture.wait_for(Duration::from_secs(10), || {
        fixture
            .block()
            .is_ok_and(|block| block["cicd"]["buildStatus"] == "passing")
    })?;
    let root = fixture.directory.path();
    let mut declarations: Value = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)?;
    declarations[0]["manifest"]["cicd"]["source"]["branch"] = json!("develop");
    declarations[1]["diagram"]["manifests"][0]["cicd"]["source"]["branch"] = json!("develop");
    write_json(root, "manifest.json", &declarations)?;
    let mut runs: Value = serde_json::from_slice(&fs::read(root.join("runs.json"))?)?;
    runs["workflow_runs"][0]["id"] = json!(101);
    runs["workflow_runs"][0]["head_branch"] = json!("develop");
    runs["workflow_runs"][0]["html_url"] = json!("https://github.com/example/api/actions/runs/101");
    runs["workflow_runs"][0]["conclusion"] = json!("failure");
    write_json(root, "runs.json", &runs)?;
    let mut jobs: Value = serde_json::from_slice(&fs::read(root.join("jobs.json"))?)?;
    jobs["jobs"][0]["run_id"] = json!(101);
    jobs["jobs"][0]["conclusion"] = json!("failure");
    write_json(root, "jobs.json", &jobs)?;
    let source = root.join("index.scry");
    fs::write(
        &source,
        format!(
            "{}\n# Subscribe to develop.\n",
            fs::read_to_string(&source)?
        ),
    )?;
    fixture.wait_for(Duration::from_secs(10), || {
        fixture.block().is_ok_and(|block| {
            block["cicd"]["buildStatus"] == "failing"
                && block["cicd"]["githubActions"]["runs"][0]["headBranch"] == "develop"
        })
    })?;
    assert_eq!(fixture.history()?["runs"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        fixture.block()?["cicd"]["githubActions"]["runs"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    // Removing an explicit branch resumes the repository default, while retaining history.
    declarations[0]["manifest"]["cicd"]["source"]["branch"] = Value::Null;
    declarations[1]["diagram"]["manifests"][0]["cicd"]["source"]["branch"] = Value::Null;
    write_json(root, "manifest.json", &declarations)?;
    fixture.observation(true)?;
    fs::write(
        &source,
        format!(
            "{}\n# Resume default branch.\n",
            fs::read_to_string(&source)?
        ),
    )?;
    fixture.wait_for(Duration::from_secs(10), || {
        fixture.block().is_ok_and(|block| {
            block["cicd"]["source"]["branch"].is_null()
                && block["cicd"]["buildStatus"] == "passing"
                && block["cicd"]["githubActions"]["runs"]
                    .as_array()
                    .map(Vec::len)
                    == Some(1)
        })
    })?;
    assert_eq!(fixture.history()?["runs"].as_array().map(Vec::len), Some(2));
    Ok(())
}
