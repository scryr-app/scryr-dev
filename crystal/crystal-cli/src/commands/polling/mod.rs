//! General background provider scheduling, independent of any one integration.
mod client;
mod gh;
mod github;

use crate::args::{GithubSyncArgs, ServerArgs, SyncArgs, SyncCommand};
use client::Client;
use gh::Gh;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::watch;

/// Dispatch a one-shot collection without installing a daemon or changing source.
pub(crate) async fn run(args: SyncArgs) -> Result<(), String> {
    match args.command {
        SyncCommand::Github(args) => sync_github(args).await,
    }
}

/// Resolve source selection once; collection itself uses only native Rust and gh.
async fn sync_github(args: GithubSyncArgs) -> Result<(), String> {
    let endpoint = args
        .common
        .graphql_url
        .clone()
        .unwrap_or_else(super::generate::upload::default_local_graphql_url);
    let org = args.common.clerk_org_id.clone();
    let envelope = tokio::task::spawn_blocking(move || {
        let project = super::workflow::Project::new(args.common)?;
        serde_json::from_str::<Value>(&project.json()?).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    let manifests = envelope["manifests"]
        .as_array()
        .ok_or("Missing manifest declarations")?;
    let selected: Vec<_> = manifests
        .iter()
        .filter(|manifest| {
            args.manifest
                .as_deref()
                .is_none_or(|selector| super::report_config::matches(manifest, selector))
        })
        .cloned()
        .collect();
    if selected.is_empty() {
        return Err("No manifest matches the sync selector".into());
    }
    if args.manifest.is_some() && selected.len() > 1 {
        return Err("Manifest selector is ambiguous; use a stable manifest ID".into());
    }
    let targets = github::targets(&selected)?;
    let client = Client::new(&endpoint, org).await?;
    let result = github::collect(&client, &Gh::default(), &targets).await?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string(&result).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "GitHub sync: {} workflows, {} observations added or updated",
            result.workflows, result.recorded
        );
    }
    Ok(())
}

/// Serve owns this future: dropping it cancels collection and any active gh child.
pub(super) async fn serve(
    args: &ServerArgs,
    mut maps: watch::Receiver<Vec<String>>,
    endpoint: &str,
) {
    if args.no_poll
        || args
            .auth_mode
            .unwrap_or(crystal_server::state::AuthMode::Local)
            != crystal_server::state::AuthMode::Local
    {
        std::future::pending::<()>().await;
        return;
    }
    let client = match Client::new(endpoint, None).await {
        Ok(client) => client,
        Err(error) => {
            eprintln!("Background providers could not start: {error}");
            return;
        }
    };
    let mut failures: u32 = 0;
    loop {
        let identifiers = maps.borrow_and_update().clone();
        if !identifiers.is_empty() {
            match cycle(&client, &identifiers).await {
                Ok(()) => failures = 0,
                Err(error) => {
                    failures = failures.saturating_add(1);
                    eprintln!("Background provider collection: {error}");
                }
            }
        }
        tokio::select! {
            () = tokio::time::sleep(retry_delay(args.poll, failures)) => {},
            changed = maps.changed() => {
                if changed.is_err() { return; }
                failures = 0;
            }
        }
    }
}

/// Each provider consumes the same current declarations; later providers join here.
async fn cycle(client: &Client, identifiers: &[String]) -> Result<(), String> {
    let manifests = client.manifests(identifiers).await?;
    let targets = github::targets(&manifests)?;
    let summary = github::collect(client, &Gh::default(), &targets).await?;
    if !targets.is_empty() {
        eprintln!(
            "GitHub sync: {} workflows, {} observations added or updated",
            summary.workflows, summary.recorded
        );
    }
    Ok(())
}

/// Failed cycles back off without overlapping work; successful cycles restore cadence.
fn retry_delay(interval: u64, failures: u32) -> Duration {
    Duration::from_secs(interval.saturating_mul(1_u64 << failures.min(8)).min(3600))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_is_bounded_and_resets_to_the_requested_interval() {
        assert_eq!(retry_delay(300, 0), Duration::from_mins(5));
        assert_eq!(retry_delay(300, 1), Duration::from_mins(10));
        assert_eq!(retry_delay(300, 100), Duration::from_hours(1));
        assert_eq!(retry_delay(15, 0), Duration::from_secs(15));
    }
}
