//! Offline collection tests exercise the real gh subprocess and HTTP boundaries.
#![allow(clippy::missing_docs_in_private_items)]
use super::*;

fn manifest(id: &str) -> Value {
    json!({"manifestId":id,"github":{"repoUrl":"https://github.com/example/api"},
        "cicd":{"source":{"workflows":["ci.yml"],"branch":"main"}}})
}

#[test]
fn selection_is_explicit_shared_and_validated() -> Result<(), String> {
    assert!(targets(&[json!({"github":{"repoUrl":"https://github.com/example/api"}})])?.is_empty());
    assert!(targets(&[json!({"cicd":{"source":{"branch":"main"}}})])?.is_empty());
    let selected = targets(&[manifest("api"), manifest("web"), manifest("api")])?;
    assert_eq!(selected.len(), 1);
    assert_eq!(selected.values().next().map(BTreeSet::len), Some(2));
    for name in [
        "../ci.yml",
        "https://evil/ci.yml",
        "ci.yml?x=1",
        "",
        "ci",
        "-ci.yml",
        "a..yml",
    ] {
        let mut input = manifest("api");
        input["cicd"]["source"]["workflows"] = json!([name]);
        assert!(targets(&[input]).is_err(), "{name}");
    }
    for id in [Value::Null, json!("")] {
        let mut input = manifest("api");
        input["manifestId"] = id;
        assert!(targets(&[input]).is_err());
    }
    let mut input = manifest("api");
    input["cicd"]["source"]["workflow_id"] = json!(42);
    assert!(targets(&[input.clone()]).is_err());
    input["cicd"]["source"]["workflows"] = json!([]);
    assert_eq!(
        targets(&[input])?
            .keys()
            .next()
            .map(|s| s.workflow.as_str()),
        Some("42")
    );
    Ok(())
}

#[test]
fn repository_urls_cannot_escape_the_selected_github_host() -> Result<(), String> {
    for value in [
        "http://github.com/a/b",
        "https://example.com/a/b",
        "https://token@github.com/a/b",
        "https://github.com/a/b?x=y",
        "https://github.com/a/b#main",
        "https://github.com/a/b/tree/main",
        "https://github.com/a/%2e%2e",
        "https://github.com/a/b:bad",
    ] {
        assert!(repository(value).is_err(), "{value}");
    }
    assert_eq!(
        repository("https://github.com/example/api.git/")?,
        "example/api"
    );
    Ok(())
}

fn run() -> Value {
    json!({"id":100,"workflow_id":42,"name":"CI","run_attempt":1,"head_branch":"main",
        "head_sha":"abcdef","html_url":"https://github.com/example/api/actions/runs/100",
        "status":"completed","conclusion":"success","created_at":"2026-09-08T10:00:00Z",
        "updated_at":"2026-09-08T10:02:00Z","repository":{"id":123}})
}

fn job(id: u64) -> Value {
    json!({"id":id,"name":"Tests","run_id":100,"run_attempt":1,
        "status":"completed","conclusion":"success","started_at":"2026-09-08T10:00:00Z",
        "completed_at":"2026-09-08T10:02:00Z","html_url":format!("https://github.com/example/api/actions/runs/100/jobs/{id}")})
}

#[test]
fn completed_cache_does_not_hide_active_jobs_or_new_attempts() -> Result<(), String> {
    let mut observed = super::super::super::report::observation(
        &json!({"repository":{"id":123,"full_name":"example/api"},"workflow_run":run()}),
    )?;
    observed.jobs = Some(Vec::new());
    let history = GithubActionsLog {
        runs: vec![observed.clone()],
    };
    assert!(unchanged(&history, &observed));
    observed.run_attempt = 2;
    assert!(!unchanged(&history, &observed));
    observed.run_attempt = 1;
    observed.status = "in_progress".into();
    observed.conclusion = None;
    let active = GithubActionsLog {
        runs: vec![observed.clone()],
    };
    assert!(!unchanged(&active, &observed));
    Ok(())
}

#[cfg(unix)]
fn fake_gh() -> Result<(tempfile::TempDir, Gh), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir()?;
    let executable = dir.path().join("gh");
    // The generated directory is quoted, and no provider content is shell code.
    let quoted = dir.path().display().to_string().replace('\'', "'\\''");
    std::fs::write(
        &executable,
        format!(
            r#"#!/bin/sh
set -eu
cd '{quoted}'
test "$1" = api
test "$2" = --hostname
test "$3" = github.com
test "$4" = --method
test "$5" = GET
printf '%s\n' "$6" >> calls
if test -f fail; then echo 'HTTP 429 rate limit (redact-me)' >&2; exit 1; fi
case "$6" in
  repos/example/api) cat repository.json ;;
  repos/example/api/actions/workflows/ci.yml) cat workflow.json ;;
  repos/example/api/actions/workflows/ci.yml/runs\?*page=2) cat runs2.json ;;
  repos/example/api/actions/workflows/ci.yml/runs\?*) cat runs.json ;;
  repos/example/api/actions/runs/100/attempts/1) cat attempt.json ;;
  *jobs\?per_page=100\&page=1) cat jobs1.json ;;
  *jobs\?per_page=100\&page=2) cat jobs2.json ;;
  *) exit 2 ;;
esac
"#
        ),
    )?;
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))?;
    for (name, value) in [
        (
            "workflow.json",
            json!({"id":42,"path":".github/workflows/ci.yml"}),
        ),
        (
            "repository.json",
            json!({"id":123,"full_name":"example/api","default_branch":"main"}),
        ),
        ("runs.json", json!({"workflow_runs":[run()]})),
        ("jobs1.json", json!({"total_count":2,"jobs":[job(1)]})),
        ("jobs2.json", json!({"total_count":2,"jobs":[job(2)]})),
    ] {
        std::fs::write(dir.path().join(name), value.to_string())?;
    }
    Ok((dir, Gh { executable }))
}

#[cfg(unix)]
#[tokio::test]
async fn catch_up_pages_and_old_active_attempts_are_refreshed()
-> Result<(), Box<dyn std::error::Error>> {
    let (dir, gh) = fake_gh()?;
    let mut saved = super::super::super::report::observation(
        &json!({"repository":{"id":123,"full_name":"example/api"},"workflow_run":run()}),
    )?;
    saved.run_id = 110;
    let id = "api".to_owned();
    let selection = Selection {
        repository: "example/api".into(),
        branch: Some("main".into()),
        workflow: "ci.yml".into(),
    };
    let mut history = BTreeMap::from([(
        &id,
        GithubActionsLog {
            runs: vec![saved.clone()],
        },
    )]);
    let page: Vec<_> = (111..=120)
        .rev()
        .map(|id| {
            let mut run = run();
            run["id"] = json!(id);
            run
        })
        .collect();
    std::fs::write(
        dir.path().join("runs.json"),
        json!({"workflow_runs":page}).to_string(),
    )?;
    let mut known = run();
    known["id"] = json!(110);
    std::fs::write(
        dir.path().join("runs2.json"),
        json!({"workflow_runs":[known]}).to_string(),
    )?;
    assert_eq!(
        recent_runs(&gh, &selection, "main", 42, &history)
            .await?
            .len(),
        11
    );
    assert!(std::fs::read_to_string(dir.path().join("calls"))?.contains("page=2"));
    saved.run_id = 100;
    saved.status = "in_progress".into();
    saved.conclusion = None;
    history
        .get_mut(&id)
        .ok_or("missing history")?
        .runs
        .push(saved);
    let mut current = run();
    current["id"] = json!(110);
    std::fs::write(
        dir.path().join("runs.json"),
        json!({"workflow_runs":[current]}).to_string(),
    )?;
    std::fs::write(dir.path().join("attempt.json"), run().to_string())?;
    let refreshed = recent_runs(&gh, &selection, "main", 42, &history).await?;
    assert_eq!(refreshed.len(), 2);
    assert!(
        refreshed
            .iter()
            .any(|run| run["id"] == 100 && run["status"] == "completed")
    );
    assert!(std::fs::read_to_string(dir.path().join("calls"))?.contains("runs/100/attempts/1"));
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn older_completed_reruns_are_found_beyond_a_known_first_page()
-> Result<(), Box<dyn std::error::Error>> {
    let (dir, gh) = fake_gh()?;
    let mut saved = super::super::super::report::observation(
        &json!({"repository":{"id":123,"full_name":"example/api"},"workflow_run":run()}),
    )?;
    saved.run_id = 110;
    let mut older = saved.clone();
    older.run_id = 100;
    let id = "api".to_owned();
    let history = BTreeMap::from([(
        &id,
        GithubActionsLog {
            runs: vec![saved, older],
        },
    )]);
    let selection = Selection {
        repository: "example/api".into(),
        branch: Some("main".into()),
        workflow: "ci.yml".into(),
    };
    let page: Vec<_> = (101..=110)
        .rev()
        .map(|id| {
            let mut value = run();
            value["id"] = json!(id);
            value
        })
        .collect();
    std::fs::write(
        dir.path().join("runs.json"),
        json!({"workflow_runs":page}).to_string(),
    )?;
    let mut rerun = run();
    rerun["run_attempt"] = json!(2);
    std::fs::write(
        dir.path().join("runs2.json"),
        json!({"workflow_runs":[rerun]}).to_string(),
    )?;
    let refreshed = recent_runs(&gh, &selection, "main", 42, &history).await?;
    assert!(
        refreshed
            .iter()
            .any(|run| run["id"] == 100 && run["run_attempt"] == 2)
    );
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn dropping_collection_terminates_the_gh_process() -> Result<(), Box<dyn std::error::Error>> {
    let (dir, gh) = fake_gh()?;
    let pid_file = dir.path().join("pid");
    std::fs::write(
        &gh.executable,
        format!(
            "#!/bin/sh\necho $$ > '{}'\nexec sleep 30\n",
            pid_file.display()
        ),
    )?;
    let collection = tokio::spawn(async move { gh.get("repos/example/api").await });
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while !pid_file.exists() {
        if tokio::time::Instant::now() > deadline {
            return Err("gh subprocess did not start".into());
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let pid: u32 = std::fs::read_to_string(&pid_file)?.trim().parse()?;
    collection.abort();
    assert!(collection.await.is_err());
    loop {
        if !std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stderr(std::process::Stdio::null())
            .status()?
            .success()
        {
            break;
        }
        if tokio::time::Instant::now() > deadline {
            return Err("cancelled gh process is still alive".into());
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn subprocess_reports_missing_auth_invalid_json_and_paginated_jobs()
-> Result<(), Box<dyn std::error::Error>> {
    let (dir, gh) = fake_gh()?;
    let observed = super::super::super::report::observation(
        &json!({"repository":{"id":123,"full_name":"example/api"},"workflow_run":run()}),
    )?;
    let payload = jobs(&gh, "example/api", &observed).await?;
    assert_eq!(payload["jobs"].as_array().map(Vec::len), Some(2));
    let mut observed = observed;
    super::super::super::report::attach_jobs(&mut observed, &payload)?;
    assert_eq!(observed.jobs.as_ref().map(Vec::len), Some(2));
    std::fs::write(
        dir.path().join("jobs2.json"),
        json!({"total_count":2,"jobs":[job(1)]}).to_string(),
    )?;
    assert!(jobs(&gh, "example/api", &observed).await.is_err());
    std::fs::write(dir.path().join("repository.json"), "bad JSON")?;
    assert!(gh.get("repos/example/api").await.is_err());
    std::fs::write(dir.path().join("fail"), "")?;
    let error = gh
        .get("repos/example/api")
        .await
        .err()
        .ok_or("expected failure")?;
    assert!(error.contains("rate limit"));
    assert!(!error.contains("redact-me"));
    let missing = Gh {
        executable: dir.path().join("absent"),
    };
    assert!(
        missing
            .get("repos/example/api")
            .await
            .err()
            .is_some_and(|e| e.contains("gh auth login"))
    );
    Ok(())
}

#[derive(Default)]
struct State {
    histories: BTreeMap<String, GithubActionsLog>,
    errors: BTreeMap<String, Value>,
    observations: usize,
}

async fn graphql(
    state: actix_web::web::Data<std::sync::Mutex<State>>,
    body: actix_web::web::Json<Value>,
) -> actix_web::HttpResponse {
    let Ok(mut state) = state.lock() else {
        return actix_web::HttpResponse::InternalServerError().finish();
    };
    let query = body["query"].as_str().unwrap_or_default();
    let id = body["variables"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let data = if query.contains("actionHistory") {
        json!({"actionHistory":state.histories.entry(id).or_default()})
    } else if query.contains("recordProviderSync") {
        state.errors.insert(id, body["variables"]["error"].clone());
        json!({"recordProviderSync":true})
    } else {
        let Ok(run) = serde_json::from_value::<GithubActionRun>(body["variables"]["run"].clone())
        else {
            return actix_web::HttpResponse::BadRequest().finish();
        };
        assert_eq!(
            body["query"].as_str().map(|s| s.contains("source:\"api\"")),
            Some(true)
        );
        state.histories.entry(id).or_default().merge(run);
        state.observations += 1;
        json!({"recordActionRun":true})
    };
    actix_web::HttpResponse::Ok().json(json!({"data":data}))
}

#[cfg(unix)]
#[actix_web::test]
async fn collect_shares_requests_preserves_history_and_recovers_after_failure()
-> Result<(), Box<dyn std::error::Error>> {
    let state = actix_web::web::Data::new(std::sync::Mutex::new(State::default()));
    let app_state = state.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("http://{}/graphql", listener.local_addr()?);
    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(app_state.clone())
            .route("/graphql", actix_web::web::post().to(graphql))
    })
    .listen(listener)?
    .run();
    let handle = server.handle();
    let task = actix_web::rt::spawn(server);
    let client = Client::new(&endpoint, None).await?;
    let (dir, gh) = fake_gh()?;
    let targets = targets(&[manifest("api"), manifest("web")])?;
    let result = collect(&client, &gh, &targets).await?;
    assert_eq!(result.workflows, 1);
    assert_eq!(result.recorded, 2);
    let calls = std::fs::read_to_string(dir.path().join("calls"))?;
    assert_eq!(calls.lines().count(), 5); // repository + workflow + runs + two job pages, shared by both blocks
    assert_eq!(collect(&client, &gh, &targets).await?.recorded, 0);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("calls"))?
            .lines()
            .count(),
        8
    );
    std::fs::write(dir.path().join("fail"), "")?;
    assert!(collect(&client, &gh, &targets).await.is_err());
    {
        let state = state.lock().map_err(|e| e.to_string())?;
        assert_eq!(state.observations, 2);
        assert!(state.errors.values().all(Value::is_string));
        assert_eq!(state.histories.len(), 2);
        drop(state);
    }
    std::fs::remove_file(dir.path().join("fail"))?;
    collect(&client, &gh, &targets).await?;
    assert!(
        state
            .lock()
            .map_err(|e| e.to_string())?
            .errors
            .values()
            .all(Value::is_null)
    );
    handle.stop(true).await;
    task.await??;
    Ok(())
}
