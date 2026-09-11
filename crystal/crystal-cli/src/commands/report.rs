//! Native `workflow_run` event reporting. No Python runtime is needed.
use crate::args::ReportArgs;
use crystal_core::action_history::GithubActionRun;
use serde_json::{Value, json};

/// Parse GitHub's event into the existing validated wire model.
fn observation(event: &Value) -> Result<GithubActionRun, String> {
    let run = event
        .get("workflow_run")
        .ok_or("expected a workflow_run event")?;
    let repo = &event["repository"];
    let url = url::Url::parse(run["html_url"].as_str().ok_or("missing run URL")?)
        .map_err(|error| error.to_string())?;
    let mapped = json!({
        "host": url.host_str(), "repositoryId": repo["id"],
        "repository": repo["full_name"], "workflowId": run["workflow_id"],
        "workflowName": run["name"], "runId": run["id"],
        "runAttempt": run["run_attempt"], "headBranch": run["head_branch"],
        "headSha": run["head_sha"], "htmlUrl": run["html_url"],
        "status": run["status"], "conclusion": run["conclusion"],
        "createdAt": run["created_at"], "updatedAt": run["updated_at"],
        "runStartedAt": run["run_started_at"], "logsUrl": run["logs_url"]
    });
    let observation: GithubActionRun =
        serde_json::from_value(mapped).map_err(|error| error.to_string())?;
    observation.validate()?;
    Ok(observation)
}

/// Restrict credential-bearing requests to TLS or loopback development servers.
pub(super) fn endpoint(value: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|error| error.to_string())?;
    if !url.username().is_empty()
        || url.password().is_some()
        || !(url.scheme() == "https"
            || (url.scheme() == "http"
                && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))))
    {
        return Err("endpoint must use HTTPS (HTTP is allowed on loopback only)".into());
    }
    Ok(url)
}

/// Select a real repository branch, not merely a branch seen in recent runs.
async fn branch(
    client: &reqwest::Client,
    args: &ReportArgs,
    event: &Value,
    run: &GithubActionRun,
    configured_token: Option<&str>,
) -> Result<String, String> {
    if let Some(branch) = &args.branch {
        return Ok(branch.clone());
    }
    let token = configured_token.map(str::to_owned).or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .ok_or("Configure GitHubAuthentication in scryr.secrets.toml for automatic branch selection, or pass --branch")?;
    let api = endpoint(
        &std::env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".into()),
    )?;
    for candidate in ["main", "master"] {
        let mut url = api.clone();
        url.path_segments_mut()
            .map_err(|()| "invalid GitHub API URL")?
            .pop_if_empty()
            .push("repos")
            .extend(run.repository.split('/'))
            .push("branches")
            .push(candidate);
        let response = client
            .get(url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|error| error.to_string())?;
        if response.status().is_success() {
            return Ok(candidate.into());
        }
        if response.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(format!(
                "GitHub branch lookup returned {}",
                response.status()
            ));
        }
    }
    event["repository"]["default_branch"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "missing repository default branch".into())
}

/// Resolve one typed pipeline and its private authentication before reading the event.
fn resolve_configuration(
    args: &ReportArgs,
) -> Result<(ReportArgs, Option<String>, Option<String>), String> {
    let mut resolved = args.clone();
    let mut configured_token = None;
    let mut repository = None;
    if let Some((manifest, root)) = super::report_config::resolve(&args.source, &args.manifest_id)?
    {
        manifest["manifestId"]
            .as_str()
            .ok_or("Missing manifest ID")?
            .clone_into(&mut resolved.manifest_id);
        let pipelines: Vec<_> = manifest["cards"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| {
                c["kind"] == "github_actions_pipeline"
                    && args
                        .card
                        .as_deref()
                        .is_none_or(|id| c["id"].as_str() == Some(id))
            })
            .collect();
        if args.card.is_some() && pipelines.is_empty() {
            return Err("No GitHub Actions pipeline matches --card".into());
        }
        if pipelines.len() > 1 {
            return Err("Multiple GitHub Actions pipelines found; select --card".into());
        }
        if let Some(card) = pipelines.first() {
            repository = card["integration"]["repository"]
                .as_str()
                .map(str::to_owned);
            resolved.workflow_id = resolved.workflow_id.or_else(|| card["workflowId"].as_u64());
            resolved.branch = resolved
                .branch
                .or_else(|| card["branch"].as_str().map(str::to_owned));
            if let Some(name) = card["integration"]["authentication"]["id"].as_str() {
                use crystal_core::integration_secrets::{
                    Authentication, SecretsFile, secrets_path,
                };
                let secrets = SecretsFile::read(&secrets_path(&root.join("index.scry")))?;
                let scope = secrets.scope(args.clerk_org_id.as_deref().unwrap_or("local-dev-org"));
                match scope.authentication.get(name) {
                    Some(Authentication::GitHub(secret)) => {
                        configured_token = Some(secret.token.clone());
                    }
                    _ => {
                        return Err(format!(
                            "Missing GitHub authentication.{name} in scryr.secrets.toml"
                        ));
                    }
                }
            }
        }
        let source = &manifest["cicd"]["source"];
        if resolved.jobs_file.is_none() {
            resolved.jobs_file = source["jobs_file"].as_str().map(|p| root.join(p));
        }
        if resolved.workflow_id.is_none() {
            resolved.workflow_id = source["workflow_id"].as_u64();
        }
        if resolved.branch.is_none() {
            resolved.branch = source["branch"].as_str().map(str::to_owned);
        }
    }
    if args.card.is_some() && repository.is_none() {
        return Err(
            "--card requires a manifest declaration selected with --path or --manifest".into(),
        );
    }
    Ok((resolved, configured_token, repository))
}

/// Send one observation through Crystal's authenticated history mutation.
pub(crate) async fn run(args: &ReportArgs) -> Result<(), String> {
    let (resolved, configured_token, repository) = resolve_configuration(args)?;
    let args = &resolved;
    crystal_core::reports::validate_manifest_id(&args.manifest_id)?;
    let event: Value = serde_json::from_slice(
        &std::fs::read(&args.event_file).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut observation = observation(&event)?;
    if repository
        .as_deref()
        .is_some_and(|repo| repo != observation.repository)
    {
        return Err("Workflow event repository does not match the selected integration".into());
    }
    if let Some(file) = &args.jobs_file {
        let payload: Value =
            serde_json::from_slice(&std::fs::read(file).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        attach_jobs(&mut observation, &payload)?;
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("scryr-action-reporter")
        .build()
        .map_err(|error| error.to_string())?;
    if args
        .workflow_id
        .is_some_and(|id| id != observation.workflow_id)
    {
        println!("Skipped unmatched workflow");
        return Ok(());
    }
    let selected = branch(
        &client,
        args,
        &event,
        &observation,
        configured_token.as_deref(),
    )
    .await?;
    if observation.head_branch.as_deref() != Some(selected.as_str()) {
        println!("Skipped run outside selected branch {selected}");
        return Ok(());
    }
    if args.dry_run {
        println!(
            "{}",
            json!({"manifestId":args.manifest_id,"run":observation})
        );
        return Ok(());
    }
    let mut request = client.post(endpoint(&args.endpoint)?).json(&json!({
        "query": "mutation Record($manifestId: String!, $run: JSON!, $source: String!) { recordActionRun(manifestId: $manifestId, run: $run, source: $source) }",
        "variables": {"manifestId": args.manifest_id, "run": observation, "source": "webhook"}
    }));
    if let Some(token) = reporting_token(&args.endpoint).await? {
        request = request.bearer_auth(token);
    }
    if let Some(org) = &args.clerk_org_id {
        request = request.header("X-Scryr-Clerk-Org-Id", org);
    }
    let response = request.send().await.map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Crystal returned {}", response.status()));
    }
    let body: Value = response.json().await.map_err(|error| error.to_string())?;
    if body["errors"]
        .as_array()
        .is_some_and(|errors| !errors.is_empty())
    {
        return Err("Crystal rejected the action observation".into());
    }
    match body["data"]["recordActionRun"].as_bool() {
        Some(true) => {
            if args.json {
                println!("{}", json!({"recorded":true}));
            } else {
                println!("Recorded action status");
            }
        }
        Some(false) => {
            if args.json {
                println!("{}", json!({"recorded":false}));
            } else {
                println!("Action status already recorded");
            }
        }
        None => return Err("Crystal returned an invalid reporting response".into()),
    }
    Ok(())
}

/// Share environment credentials and cached CLI login across reporters.
pub(super) async fn reporting_token(target: &str) -> Result<Option<String>, String> {
    if let Ok(token) = std::env::var("SCRYR_TOKEN") {
        return Ok(Some(token));
    }
    let url = endpoint(target)?;
    if matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")) {
        return Ok(None);
    }
    crate::auth::access_token().await.map(Some)
}

/// Attach a complete, correctly scoped GitHub jobs response.
fn attach_jobs(run: &mut GithubActionRun, payload: &Value) -> Result<(), String> {
    let jobs = payload["jobs"]
        .as_array()
        .ok_or("jobs file must contain a GitHub jobs response")?;
    if payload["total_count"].as_u64() != Some(jobs.len() as u64) {
        return Err("jobs file is incomplete; combine all pages before reporting".into());
    }
    let mut parsed = Vec::new();
    for job in jobs {
        if job["run_id"].as_u64() != Some(run.run_id)
            || job["run_attempt"].as_u64() != Some(run.run_attempt)
        {
            return Err("job belongs to a different workflow run or attempt".into());
        }
        parsed.push(serde_json::from_value(json!({
            "id":job["id"], "name":job["name"], "status":job["status"], "conclusion":job["conclusion"],
            "startedAt":job["started_at"], "completedAt":job["completed_at"], "htmlUrl":job["html_url"]
        })).map_err(|e| e.to_string())?);
    }
    run.jobs = Some(parsed);
    run.validate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_workflow_event_and_summarizes_latest_build() -> Result<(), String> {
        let event = json!({"repository": {"id": 123, "full_name": "example/api"}, "workflow_run": {
            "id": 100, "workflow_id": 42, "name": "CI", "run_attempt": 1,
            "head_branch": "main", "head_sha": "abc", "html_url": "https://github.com/example/api/actions/runs/100",
            "status": "completed", "conclusion": "success",
            "created_at": "2026-09-08T10:00:00Z", "updated_at": "2026-09-08T10:02:00Z"
        }});
        let first = observation(&event)?;
        let mut log = crystal_core::action_history::GithubActionsLog::default();
        log.merge(first.clone());
        assert_eq!(log.build_status(), Some("passing"));
        let mut second = first;
        second.run_id = 101;
        second.workflow_id = 99;
        second.conclusion = Some("failure".into());
        log.merge(second.clone());
        assert_eq!(log.build_status(), Some("failing"));
        second.run_attempt = 2;
        second.status = "in_progress".into();
        second.conclusion = None;
        log.merge(second.clone());
        assert_eq!(log.build_status(), Some("pending"));
        second.status = "completed".into();
        second.conclusion = Some("cancelled".into());
        second.run_attempt = 3;
        log.merge(second);
        assert_eq!(log.build_status(), None);
        Ok(())
    }

    #[test]
    fn rejects_non_workflow_payload_and_insecure_endpoints() {
        assert!(observation(&json!({})).is_err());
        assert!(endpoint("http://example.com/graphql").is_err());
        assert!(endpoint("https://user:secret@example.com/graphql").is_err());
        assert!(endpoint("http://127.0.0.1:8000/graphql").is_ok());
    }
}
