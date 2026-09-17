//! General background provider scheduling, independent of any one integration.
mod client;
mod dependencies;
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
    let client = Client::new(&endpoint, org).await?;
    let result = collect_all(&client, &selected).await?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string(&result).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "GitHub sync: {} workflows, {} workflow observations; {} dependency repositories, {} snapshots",
            result.workflows,
            result.recorded,
            result.dependency_repositories,
            result.dependency_snapshots
        );
    }
    Ok(())
}

/// Serve owns this future: dropping it cancels collection and any active gh child.
pub(super) async fn serve(args: &ServerArgs, maps: watch::Receiver<Vec<String>>, endpoint: &str) {
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
    tokio::join!(
        schedule(&client, maps.clone(), args.poll, Provider::Actions),
        schedule(&client, maps.clone(), args.poll, Provider::Inventory),
        schedule(&client, maps, args.poll, Provider::Security),
    );
}

/// Independent schedules keep unavailable providers from delaying successful ones.
#[derive(Clone, Copy, Debug)]
enum Provider {
    /// Workflow and job observations.
    Actions,
    /// Repository dependency inventory.
    Inventory,
    /// Open Dependabot security findings.
    Security,
}

/// One non-overlapping loop and exponential retry budget per provider.
async fn schedule(
    client: &Client,
    mut maps: watch::Receiver<Vec<String>>,
    interval: u64,
    provider: Provider,
) {
    let mut failures: u32 = 0;
    let mut pending_exports = dependencies::PendingExports::default();
    loop {
        let identifiers = maps.borrow_and_update().clone();
        if !identifiers.is_empty() {
            match cycle(client, &identifiers, provider, &mut pending_exports).await {
                Ok(()) => failures = 0,
                Err(error) => {
                    failures = failures.saturating_add(1);
                    eprintln!("Background {provider:?} collection: {error}");
                }
            }
        }
        tokio::select! {
            () = tokio::time::sleep(retry_delay(interval, failures)) => {},
            changed = maps.changed() => {
                if changed.is_err() { return; }
                failures = 0;
            }
        }
    }
}

/// Providers consume the same current declarations, with separate failure lifecycles.
async fn cycle(
    client: &Client,
    identifiers: &[String],
    provider: Provider,
    pending_exports: &mut dependencies::PendingExports,
) -> Result<(), String> {
    let manifests = client.manifests(identifiers).await?;
    let gh = Gh::default();
    match provider {
        Provider::Actions => {
            let targets = github::targets(&manifests)?;
            let result = github::collect(client, &gh, &targets).await?;
            if !targets.is_empty() {
                eprintln!(
                    "GitHub sync: {} workflows, {} observations added or updated",
                    result.workflows, result.recorded
                );
            }
        }
        Provider::Inventory | Provider::Security => {
            let targets = dependencies::component_targets(
                &manifests,
                matches!(provider, Provider::Inventory),
            )?;
            let result = dependencies::collect(client, &gh, &targets, pending_exports).await?;
            if !targets.is_empty() {
                eprintln!(
                    "GitHub {provider:?} sync: {} repositories, {} snapshots",
                    result.repositories, result.snapshots
                );
            }
        }
    }
    Ok(())
}

/// Keep existing one-shot JSON summary fields compatible while adding provider totals.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Summary {
    /// Workflow selections successfully collected.
    workflows: usize,
    /// New or enriched workflow observations.
    recorded: usize,
    /// Repositories requesting dependency data.
    dependency_repositories: usize,
    /// Dependency component snapshots written.
    dependency_snapshots: usize,
}

/// Independent provider errors must not prevent other configured collections.
async fn collect_all(client: &Client, manifests: &[Value]) -> Result<Summary, String> {
    let gh = Gh::default();
    let actions = match github::targets(manifests) {
        Ok(targets) => github::collect(client, &gh, &targets).await,
        Err(error) => Err(error),
    };
    let dependencies = match dependencies::targets(manifests) {
        Ok(targets) => {
            dependencies::collect(
                client,
                &gh,
                &targets,
                &mut dependencies::PendingExports::default(),
            )
            .await
        }
        Err(error) => Err(error),
    };
    match (actions, dependencies) {
        (Ok(actions), Ok(dependencies)) => Ok(Summary {
            workflows: actions.workflows,
            recorded: actions.recorded,
            dependency_repositories: dependencies.repositories,
            dependency_snapshots: dependencies.snapshots,
        }),
        (Err(a), Err(d)) => Err(format!("{a}; {d}")),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
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
