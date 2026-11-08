//! Collect delivery evidence through GitHub CLI without executing project tasks.
#![allow(clippy::missing_docs_in_private_items)]
mod gh;
mod normalize;

use crate::args::DeliveryArgs;
use crystal_core::delivery::{DeliverySnapshot, GitHubDelivery};
use crystal_core::reports::{Report, ReportData};
use serde_json::{Value, json};

pub(super) async fn run(args: &DeliveryArgs) -> Result<(), String> {
    let mut source = args.source.clone();
    if source.path.is_none() {
        source.path = Some("index.scry".into());
    }
    let (manifest, _) = super::super::report_config::resolve(&source, &args.manifest_id)?
        .ok_or("Delivery collection requires an index.scry configuration")?;
    let manifest_id = manifest["manifestId"]
        .as_str()
        .ok_or("Missing manifest ID")?;
    let config: GitHubDelivery = serde_json::from_value(manifest["delivery"].clone())
        .map_err(|e| format!("Invalid or missing delivery configuration: {e}"))?;
    config.validate()?;
    let task = args.resolve_task(&config.tasks)?;
    let snapshot = if let Some(path) = &args.file {
        let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if metadata.len() > 2_000_000 {
            return Err("Delivery snapshot exceeds 2 MB".into());
        }
        serde_json::from_slice::<DeliverySnapshot>(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        let config = config.clone();
        let id = manifest_id.to_owned();
        let limit = task.limit;
        let verify = task.verify_attestations;
        tokio::task::spawn_blocking(move || normalize::collect(&config, &id, limit, verify))
            .await
            .map_err(|e| e.to_string())??
    };
    if snapshot.host != config.repository.host
        || snapshot.repository != format!("{}/{}", config.repository.owner, config.repository.name)
        || snapshot.configuration.as_ref().is_none_or(|configuration| {
            serde_json::to_value(configuration).ok() != serde_json::to_value(&config).ok()
        })
    {
        return Err("Snapshot does not match the manifest delivery configuration".into());
    }
    snapshot.validate_for_manifest(manifest_id)?;
    let report = Report {
        schema_version: 1,
        source: "github".into(),
        scope: format!("{}/{}", snapshot.host, snapshot.repository),
        run_id: snapshot.observed_at.to_rfc3339(),
        attempt: 1,
        commit_sha: None,
        branch: None,
        observed_at: snapshot.observed_at,
        report_url: Some(format!("https://{}/{}", snapshot.host, snapshot.repository)),
        data: ReportData::Delivery {
            snapshot: Box::new(snapshot),
        },
    };
    report.validate()?;
    let variables = json!({"manifestId":manifest_id,"report":report});
    if args.dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&variables).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let mut request = super::client()?.post(super::super::report::endpoint(&args.endpoint)?)
        .json(&json!({"query":"mutation Report($manifestId: String!, $report: JSON!) { recordReport(manifestId: $manifestId, report: $report) }", "variables":variables}));
    if let Some(token) = super::super::report::reporting_token(&args.endpoint).await? {
        request = request.bearer_auth(token);
    }
    if let Some(org) = &args.clerk_org_id {
        request = request.header("X-Scryr-Clerk-Org-Id", org);
    }
    let response = request.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Scryr returned {}", response.status()));
    }
    let body: Value = response.json().await.map_err(|e| e.to_string())?;
    if body.get("errors").is_some() {
        return Err(format!(
            "Scryr rejected delivery report: {}",
            body["errors"]
        ));
    }
    let recorded = body["data"]["recordReport"]
        .as_bool()
        .ok_or("Invalid reporting response")?;
    if args.json {
        println!("{}", json!({"recorded":recorded,"manifestId":manifest_id}));
    } else {
        println!(
            "{}",
            if recorded {
                "Recorded delivery snapshot"
            } else {
                "Delivery snapshot already recorded"
            }
        );
    }
    Ok(())
}
