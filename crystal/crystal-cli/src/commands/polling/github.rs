//! GitHub Actions provider: explicit declarations, shared collection, durable observations.
use super::{client::Client, gh::Gh};
use crystal_core::action_history::{GithubActionRun, GithubActionsLog};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// A workflow selection shared by one or more manifest blocks.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Selection {
    /// Validated owner/name from a canonical GitHub URL.
    pub repository: String,
    /// Explicit branch or the repository's default branch.
    pub branch: Option<String>,
    /// Workflow filename or numeric ID.
    pub workflow: String,
}

/// One request per distinct repository/branch/workflow, then fan out to blocks.
pub(super) type Targets = BTreeMap<Selection, BTreeSet<String>>;

/// Validate declarations before any credential-bearing subprocess is launched.
pub(super) fn targets(manifests: &[Value]) -> Result<Targets, String> {
    let mut result = Targets::new();
    for manifest in manifests {
        let source = &manifest["cicd"]["source"];
        if source.is_null() {
            continue;
        }
        let mut workflows = Vec::new();
        if let Some(values) = source["workflows"].as_array() {
            for value in values {
                let workflow = value
                    .as_str()
                    .ok_or("workflows must contain workflow filenames")?;
                if !valid_workflow(workflow) {
                    return Err(
                        "workflows must contain safe .yml or .yaml filenames (for example ci.yml)"
                            .into(),
                    );
                }
                workflows.push(workflow.to_owned());
            }
        }
        if let Some(id) = source["workflow_id"].as_u64() {
            if id == 0 || !workflows.is_empty() {
                return Err("Use either workflows or a positive workflow_id".into());
            }
            workflows.push(id.to_string());
        }
        // An event-only source does not implicitly subscribe to every workflow.
        if workflows.is_empty() {
            continue;
        }
        let id = manifest["manifestId"]
            .as_str()
            .ok_or("GitHub polling requires a stable manifest_id")?;
        crystal_core::reports::validate_manifest_id(id)?;
        let repo = repository(
            manifest["github"]["repoUrl"]
                .as_str()
                .ok_or("GitHub polling requires github.repo_url")?,
        )?;
        let branch = source["branch"].as_str().map(str::to_owned);
        if branch.as_ref().is_some_and(|b| b.trim().is_empty()) {
            return Err("GitHub polling branch must not be empty".into());
        }
        for workflow in workflows {
            result
                .entry(Selection {
                    repository: repo.clone(),
                    branch: branch.clone(),
                    workflow,
                })
                .or_default()
                .insert(id.to_owned());
        }
    }
    Ok(result)
}

/// Only filenames are accepted, not paths, URLs, or API query fragments.
#[allow(clippy::case_sensitive_file_extension_comparisons)] // GitHub paths and the SDK contract are case-sensitive.
fn valid_workflow(value: &str) -> bool {
    (value.ends_with(".yml") || value.ends_with(".yaml"))
        && value
            .as_bytes()
            .first()
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        && !value.contains("..")
}

/// Reject URLs that could turn gh into a request to an unintended endpoint.
fn repository(value: &str) -> Result<String, String> {
    let url = url::Url::parse(value).map_err(|_| "Invalid GitHub repository URL")?;
    let path = url.path().trim_end_matches('/').trim_end_matches(".git");
    let parts: Vec<_> = path.trim_start_matches('/').split('/').collect();
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || parts.len() != 2
        || parts.iter().any(|p| {
            p.is_empty()
                || *p == "."
                || *p == ".."
                || !p
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        })
    {
        return Err(
            "Use a canonical https://github.com/owner/repository URL for GitHub polling".into(),
        );
    }
    Ok(parts.join("/"))
}

/// Summary shared by foreground and scheduled collection.
#[derive(Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Summary {
    /// Number of distinct subscriptions collected.
    pub workflows: usize,
    /// New or enriched observations persisted across all blocks.
    pub recorded: usize,
}

/// Run every selection independently and record block-level collection health.
pub(super) async fn collect(
    client: &Client,
    gh: &Gh,
    targets: &Targets,
) -> Result<Summary, String> {
    let mut summary = Summary::default();
    let mut errors = BTreeMap::<String, Option<String>>::new();
    let mut repositories = BTreeMap::new();
    for (selection, manifests) in targets {
        let result = collect_selection(client, gh, selection, manifests, &mut repositories).await;
        if let Ok(recorded) = &result {
            summary.recorded += recorded;
            summary.workflows += 1;
        }
        for manifest in manifests {
            let error = errors.entry(manifest.clone()).or_default();
            if let Err(message) = &result {
                *error = Some(message.clone());
            }
        }
    }
    let mut failed = false;
    for (manifest, error) in errors {
        let context = targets.iter().find_map(|(selection, manifests)| {
            if !manifests.contains(&manifest) {
                return None;
            }
            repositories.get(&selection.repository).map(|metadata| {
                json!({
                    "repository":selection.repository,
                    "defaultBranch":metadata["default_branch"]
                })
            })
        });
        client
            .status(&manifest, "github", error.as_deref(), context.as_ref())
            .await?;
        if let Some(error) = error {
            failed = true;
            eprintln!("GitHub sync for {manifest}: {error}");
        }
    }
    if failed {
        return Err("GitHub collection was incomplete; previous observations are retained".into());
    }
    Ok(summary)
}

/// Fetch one workflow once, reusing repository identity within a poll cycle.
async fn collect_selection(
    client: &Client,
    gh: &Gh,
    selection: &Selection,
    manifests: &BTreeSet<String>,
    repositories: &mut BTreeMap<String, Value>,
) -> Result<usize, String> {
    let repo = &selection.repository;
    if !repositories.contains_key(repo) {
        repositories.insert(repo.clone(), gh.get(&format!("repos/{repo}")).await?);
    }
    let metadata = repositories
        .get(repo)
        .ok_or("Missing GitHub repository metadata")?;
    let branch = selection
        .branch
        .as_deref()
        .or_else(|| metadata["default_branch"].as_str())
        .ok_or("GitHub repository has no default branch")?;
    let mut histories = BTreeMap::new();
    for manifest in manifests {
        let data = client
            .execute(
                "query($id:String!){actionHistory(manifestId:$id,limit:100)}",
                json!({"id":manifest}),
            )
            .await?;
        let history: GithubActionsLog =
            serde_json::from_value(data["actionHistory"].clone()).map_err(|e| e.to_string())?;
        histories.insert(manifest, history);
    }
    let workflow = gh
        .get(&format!(
            "repos/{repo}/actions/workflows/{}",
            selection.workflow
        ))
        .await?;
    let workflow_id = workflow["id"]
        .as_u64()
        .ok_or("GitHub returned an invalid workflow ID")?;
    let workflow_path = workflow["path"]
        .as_str()
        .ok_or("GitHub returned no workflow path")?;
    let runs = recent_runs(gh, selection, branch, workflow_id, &histories).await?;
    let mut recorded = 0;
    for run in runs.iter().rev() {
        if run["head_branch"].as_str() != Some(branch)
            || run["repository"]["id"] != metadata["id"]
            || run["workflow_id"].as_u64() != Some(workflow_id)
        {
            return Err("GitHub run does not match the selected repository and branch".into());
        }
        let mut observation = super::super::report::observation(
            &json!({"repository":metadata,"workflow_run":run,"workflow":{"path":workflow_path}}),
        )?;
        if histories
            .values()
            .all(|history| unchanged(history, &observation))
        {
            continue;
        }
        let jobs = jobs(gh, repo, &observation).await?;
        super::super::report::attach_jobs(&mut observation, &jobs)?;
        for manifest in manifests {
            let data = client.execute("mutation($id:String!,$run:JSON!){recordActionRun(manifestId:$id,run:$run,source:\"api\")}",
                json!({"id":manifest,"run":observation})).await?;
            recorded += usize::from(
                data["recordActionRun"]
                    .as_bool()
                    .ok_or("Scryr returned an invalid observation result")?,
            );
        }
    }
    Ok(recorded)
}

/// Backfill ten runs initially, then scan up to 100 runs (including older reruns).
///
/// Active attempts outside that window are explicitly refreshed.
async fn recent_runs(
    gh: &Gh,
    selection: &Selection,
    branch: &str,
    workflow_id: u64,
    histories: &BTreeMap<&String, GithubActionsLog>,
) -> Result<Vec<Value>, String> {
    let known: BTreeMap<_, _> = histories
        .values()
        .flat_map(|history| &history.runs)
        .filter(|run| run.workflow_id == workflow_id && run.head_branch.as_deref() == Some(branch))
        .map(|run| ((run.run_id, run.run_attempt), run))
        .collect();
    let mut runs = Vec::new();
    for page in 1..=10 {
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("branch", branch)
            .append_pair("per_page", "10")
            .append_pair("page", &page.to_string())
            .finish();
        let payload = gh
            .get(&format!(
                "repos/{}/actions/workflows/{}/runs?{query}",
                selection.repository, selection.workflow
            ))
            .await?;
        let batch = payload["workflow_runs"]
            .as_array()
            .ok_or("GitHub returned invalid workflow runs")?;
        if batch.len() > 10 {
            return Err("GitHub exceeded the requested run page size".into());
        }
        runs.extend(batch.iter().cloned());
        if known.is_empty() || batch.len() < 10 {
            break;
        }
        if page == 10 {
            eprintln!(
                "GitHub sync reached the 100-run catch-up bound for {} / {}",
                selection.repository, selection.workflow
            );
        }
    }
    let seen: BTreeSet<_> = runs
        .iter()
        .filter_map(|run| Some((run["id"].as_u64()?, run["run_attempt"].as_u64()?)))
        .collect();
    for ((id, attempt), run) in known {
        if run.status != "completed" && !seen.contains(&(id, attempt)) {
            runs.push(
                gh.get(&format!(
                    "repos/{}/actions/runs/{id}/attempts/{attempt}",
                    selection.repository
                ))
                .await?,
            );
        }
    }
    Ok(runs)
}

/// Complete job snapshots are scoped to the exact attempt and bounded to 2,000 jobs.
async fn jobs(gh: &Gh, repo: &str, run: &GithubActionRun) -> Result<Value, String> {
    let mut jobs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut total = None;
    for page in 1..=20 {
        let payload = gh
            .get(&format!(
                "repos/{repo}/actions/runs/{}/attempts/{}/jobs?per_page=100&page={page}",
                run.run_id, run.run_attempt
            ))
            .await?;
        let count = payload["total_count"]
            .as_u64()
            .ok_or("GitHub returned an invalid job count")?;
        if total.is_some_and(|n| n != count) {
            return Err("GitHub jobs changed during pagination; retrying next collection".into());
        }
        total = Some(count);
        let batch = payload["jobs"]
            .as_array()
            .ok_or("GitHub returned invalid jobs")?;
        for job in batch {
            let id = job["id"]
                .as_u64()
                .ok_or("GitHub returned an invalid job ID")?;
            if !ids.insert(id) {
                return Err("GitHub returned duplicate jobs across pages".into());
            }
            jobs.push(job.clone());
        }
        if jobs.len() as u64 == count {
            return Ok(json!({"total_count":count,"jobs":jobs}));
        }
        if batch.is_empty() || jobs.len() as u64 > count {
            break;
        }
    }
    Err("GitHub job snapshot is incomplete or exceeds 2,000 jobs".into())
}

/// Active runs are re-read so job transitions aren't lost when run timestamps lag.
fn unchanged(history: &GithubActionsLog, run: &GithubActionRun) -> bool {
    run.status == "completed"
        && history.runs.iter().any(|saved| {
            saved.identity() == run.identity()
                && saved.status == run.status
                && saved.conclusion == run.conclusion
                && saved.updated_at == run.updated_at
                && saved.workflow_path == run.workflow_path
                && saved.jobs.is_some()
        })
}

#[cfg(test)]
mod tests;
