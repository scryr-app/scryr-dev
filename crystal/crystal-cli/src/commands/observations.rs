//! Import existing result artifacts without running project code.
#![allow(clippy::missing_docs_in_private_items)]
use crate::args::{ObservationArgs, ReportCommand, ReportsArgs};
use crystal_core::reports::{DependencyAlert, Report, ReportData};
use serde_json::{Value, json};
mod parsers;

fn required<'a>(v: Option<&'a str>, name: &str) -> Result<&'a str, String> {
    v.filter(|s| !s.is_empty())
        .ok_or_else(|| format!("--{name} is required"))
}
fn read_files(args: &ObservationArgs) -> Result<Vec<String>, String> {
    if args.file.is_empty() {
        return Err("at least one --file is required".into());
    }
    args.file
        .iter()
        .map(|p| {
            let size = std::fs::metadata(p)
                .map_err(|e| format!("{}: {e}", p.display()))?
                .len();
            if size > 20_000_000 {
                return Err("input exceeds 20 MB".into());
            }
            std::fs::read_to_string(p).map_err(|e| e.to_string())
        })
        .collect()
}
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("scryr-report")
        .build()
        .map_err(|e| e.to_string())
}

async fn dependencies(args: &ObservationArgs) -> Result<ReportData, String> {
    if args.provider != "dependabot" {
        return Err("supported provider: dependabot".into());
    }
    let repository = required(args.repository.as_deref(), "repository")?;
    let path = required(args.manifest_path.as_deref(), "manifest-path")?;
    let mut alerts = Vec::new();
    if args.file.is_empty() {
        let token = std::env::var("GITHUB_TOKEN")
            .map_err(|_| "GITHUB_TOKEN with Dependabot alerts read permission is required")?;
        let api =
            std::env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".into());
        let base = super::report::endpoint(&api)?;
        if repository.split('/').count() != 2
            || repository.split('/').any(|s| {
                s.is_empty()
                    || !s
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
            })
        {
            return Err("invalid owner/repository".into());
        }
        let c = client()?;
        // Empty terminal page proves completeness. Publish nothing if any page fails.
        for page in 1..=10000 {
            let url = format!(
                "{}/repos/{repository}/dependabot/alerts?per_page=100&page={page}",
                base.as_str().trim_end_matches('/')
            );
            let response = c
                .get(url)
                .bearer_auth(&token)
                .header("Accept", "application/vnd.github+json")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                return Err(format!(
                    "Dependabot collection failed ({}); no snapshot uploaded",
                    response.status()
                ));
            }
            let batch: Vec<Value> = response.json().await.map_err(|e| e.to_string())?;
            if batch.is_empty() {
                break;
            }
            alerts.extend(batch);
            if page == 10000 {
                return Err("Dependabot pagination limit exceeded; no snapshot uploaded".into());
            }
        }
    } else {
        for s in read_files(args)? {
            alerts.extend(serde_json::from_str::<Vec<Value>>(&s).map_err(|e| e.to_string())?);
        }
    }
    let mut normalized = Vec::new();
    for a in alerts {
        let text = |v: &Value| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "malformed Dependabot alert".to_owned())
        };
        let manifest_path = text(&a["dependency"]["manifest_path"])?;
        if manifest_path != path {
            continue;
        }
        normalized.push(DependencyAlert {
            number: a["number"].as_u64().ok_or("missing alert number")?,
            package: text(&a["dependency"]["package"]["name"])?,
            ecosystem: text(&a["dependency"]["package"]["ecosystem"])?,
            manifest_path,
            severity: text(&a["security_advisory"]["severity"])?,
            state: text(&a["state"])?,
            url: text(&a["html_url"])?,
        });
    }
    normalized.sort_by_key(|a| a.number);
    Ok(ReportData::Dependencies {
        repository: repository.into(),
        manifest_path: path.into(),
        alerts: normalized,
    })
}

pub(crate) async fn run(args: &ReportsArgs) -> Result<(), String> {
    if let ReportCommand::Actions(a) = &args.command {
        return super::report::run(a).await;
    }
    let mut resolved = args.clone();
    let observation = match &mut resolved.command {
        ReportCommand::Tests(a)
        | ReportCommand::Coverage(a)
        | ReportCommand::Dependencies(a)
        | ReportCommand::Deployment(a) => a,
        ReportCommand::Actions(_) => unreachable!(),
    };
    if let Some((manifest, root)) =
        super::report_config::resolve(&observation.source, &observation.manifest_id)?
    {
        manifest["manifestId"]
            .as_str()
            .ok_or("Missing manifest ID")?
            .clone_into(&mut observation.manifest_id);
        let source = &manifest["tests"]["source"];
        if matches!(&args.command, ReportCommand::Tests(_)) {
            if observation.file.is_empty()
                && let Some(files) = source["files"].as_array()
            {
                observation.file = files
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|p| root.join(p))
                    .collect();
            }
            if observation.format.is_none() {
                observation.format = source["format"].as_str().map(str::to_owned);
            }
            if observation.suite.is_none() {
                observation.suite = source["suite"].as_str().map(str::to_owned);
            }
        }
    }
    let args = &resolved;
    let (a, data) = match &args.command {
        ReportCommand::Tests(a) => (
            a,
            parsers::tests(&read_files(a)?, a.format.as_deref().unwrap_or("junit"))?,
        ),
        ReportCommand::Coverage(a) => (
            a,
            parsers::coverage(&read_files(a)?, required(a.format.as_deref(), "format")?)?,
        ),
        ReportCommand::Dependencies(a) => (a, dependencies(a).await?),
        ReportCommand::Deployment(a) => (
            a,
            ReportData::Deployment {
                environment: required(a.environment.as_deref(), "environment")?.into(),
                status: required(a.status.as_deref(), "status")?.into(),
                version: required(a.version.as_deref(), "version")?.into(),
            },
        ),
        ReportCommand::Actions(_) => unreachable!(),
    };
    let scope = match &data {
        ReportData::Dependencies {
            repository,
            manifest_path,
            ..
        } => format!("{repository}:{manifest_path}"),
        ReportData::Deployment { environment, .. } => environment.clone(),
        _ => a.shard.as_ref().map_or_else(
            || a.suite.as_deref().unwrap_or("default").to_owned(),
            |s| format!("{}/shard/{s}", a.suite.as_deref().unwrap_or("default")),
        ),
    };
    let report = Report {
        schema_version: 1,
        source: match &data {
            ReportData::Dependencies { .. } => "dependabot",
            ReportData::Tests { .. } => "junit",
            ReportData::Coverage { .. } => a.format.as_deref().unwrap_or("coverage"),
            ReportData::Deployment { .. } => "cli",
        }
        .into(),
        scope,
        run_id: a.run_id.clone(),
        attempt: a.attempt,
        commit_sha: a.commit_sha.clone(),
        branch: a.branch.clone(),
        observed_at: a
            .observed_at
            .parse()
            .map_err(|_| "--observed-at must be RFC3339")?,
        report_url: a.report_url.clone(),
        data,
    };
    report.validate()?;
    crystal_core::reports::validate_manifest_id(&a.manifest_id)?;
    send(a, &report).await
}

async fn send(a: &ObservationArgs, report: &Report) -> Result<(), String> {
    let payload = json!({"manifestId":a.manifest_id,"report":report});
    if a.dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let mut req=client()?.post(super::report::endpoint(&a.endpoint)?).json(&json!({"query":"mutation Report($manifestId: String!, $report: JSON!) { recordReport(manifestId: $manifestId, report: $report) }","variables":payload}));
    if let Some(token) = super::report::reporting_token(&a.endpoint).await? {
        req = req.bearer_auth(token);
    }
    if let Some(org) = &a.clerk_org_id {
        req = req.header("X-Scryr-Clerk-Org-Id", org);
    }
    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Scryr returned {}", response.status()));
    }
    let body: Value = response.json().await.map_err(|e| e.to_string())?;
    if body.get("errors").is_some() {
        return Err(format!("Scryr rejected report: {}", body["errors"]));
    }
    let recorded = body["data"]["recordReport"]
        .as_bool()
        .ok_or("invalid reporting response")?;
    if a.json {
        println!(
            "{}",
            json!({"recorded":recorded,"manifestId":a.manifest_id})
        );
    } else {
        println!(
            "{}",
            if recorded {
                "Recorded report"
            } else {
                "Report already recorded"
            }
        );
    }
    Ok(())
}
