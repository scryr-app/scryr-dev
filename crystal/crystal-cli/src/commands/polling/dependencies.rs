//! Repository-wide dependency inventory and security polling, with independent health.
mod parse;

use super::{client::Client, gh::Gh, github};
use crystal_core::github_dependencies::{
    DependencyInventory, DependencySecurity, GithubDependencySnapshot,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

/// Each requested component fans out only to subscribed stable block identities.
#[derive(Default)]
pub(super) struct Subscription {
    /// Blocks requesting repository inventory.
    inventory: BTreeSet<String>,
    /// Blocks requesting repository security alerts.
    security: BTreeSet<String>,
}

/// Selections grouped by repository, independently of workflow/branch selections.
pub(super) type Targets = BTreeMap<String, Subscription>;

/// Unfinished exports survive collection timeouts and retries within this serve session.
pub(super) type PendingExports = BTreeMap<String, String>;

/// Validate explicit dependency subscriptions before credential-bearing requests.
pub(super) fn targets(manifests: &[Value]) -> Result<Targets, String> {
    let mut targets = Targets::new();
    for manifest in manifests {
        let source = &manifest["dependencies"]["source"];
        if source.is_null() {
            continue;
        }
        if source["provider"].as_str() != Some("github") {
            return Err("Dependency source requires provider=github".into());
        }
        let flag = |name: &str| -> Result<bool, String> {
            source.get(name).map_or(Ok(true), |value| {
                value
                    .as_bool()
                    .ok_or_else(|| format!("Dependency {name} must be a boolean"))
            })
        };
        let inventory = flag("inventory")?;
        let security = flag("security")?;
        if !inventory && !security {
            return Err("Enable inventory or security for dependency polling".into());
        }
        let id = manifest["manifestId"]
            .as_str()
            .ok_or("Dependency polling requires a stable manifest_id")?;
        crystal_core::reports::validate_manifest_id(id)?;
        let repository = github::repository(
            manifest["github"]["repoUrl"]
                .as_str()
                .ok_or("Dependency polling requires github.repo_url")?,
        )?
        .to_ascii_lowercase();
        let entry = targets.entry(repository).or_default();
        if inventory {
            entry.inventory.insert(id.to_owned());
        }
        if security {
            entry.security.insert(id.to_owned());
        }
    }
    Ok(targets)
}

/// Select one independently scheduled component while retaining shared-repository fanout.
pub(super) fn component_targets(manifests: &[Value], inventory: bool) -> Result<Targets, String> {
    let mut targets = targets(manifests)?;
    targets.retain(|_, selection| {
        if inventory {
            selection.security.clear();
        } else {
            selection.inventory.clear();
        }
        !selection.inventory.is_empty() || !selection.security.is_empty()
    });
    Ok(targets)
}

/// Report repository work independently from workflow observations.
#[derive(Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Summary {
    /// Distinct repositories requested.
    pub repositories: usize,
    /// Successfully written block component snapshots.
    pub snapshots: usize,
}

/// Every repository and component is attempted, even if another provider fails.
pub(super) async fn collect(
    client: &Client,
    gh: &Gh,
    targets: &Targets,
    pending: &mut PendingExports,
) -> Result<Summary, String> {
    let mut summary = Summary {
        repositories: targets.len(),
        ..Summary::default()
    };
    pending.retain(|repository, _| targets.contains_key(repository));
    let mut failed = false;
    for (repository, subscription) in targets {
        if !subscription.inventory.is_empty() {
            let result = inventory(gh, repository, pending).await.map(|inventory| {
                GithubDependencySnapshot {
                    repository: repository.clone(),
                    inventory: Some(inventory),
                    security: None,
                }
            });
            failed |= publish(
                client,
                repository,
                "github_dependencies_inventory",
                &subscription.inventory,
                result,
                &mut summary,
            )
            .await;
        }
        if !subscription.security.is_empty() {
            let result = security(gh, repository)
                .await
                .map(|security| GithubDependencySnapshot {
                    repository: repository.clone(),
                    inventory: None,
                    security: Some(security),
                });
            failed |= publish(
                client,
                repository,
                "github_dependencies_security",
                &subscription.security,
                result,
                &mut summary,
            )
            .await;
        }
    }
    if failed {
        return Err("Dependency collection was incomplete; unavailable components retain last known observations".into());
    }
    Ok(summary)
}

/// Store health even on first failure, while never writing a failed snapshot as empty.
async fn publish(
    client: &Client,
    repository: &str,
    provider: &str,
    manifests: &BTreeSet<String>,
    snapshot: Result<GithubDependencySnapshot, String>,
    summary: &mut Summary,
) -> bool {
    let mut failed = false;
    for id in manifests {
        let result = match &snapshot {
            Ok(snapshot) => client.execute("mutation($id:String!,$snapshot:JSON!){recordGithubDependencies(manifestId:$id,snapshot:$snapshot)}", json!({"id":id,"snapshot":snapshot})).await.and_then(|data| {
                data["recordGithubDependencies"].as_bool().ok_or_else(|| "Scryr returned an invalid dependency snapshot result".into())
            }),
            Err(error) => Err(error.clone()),
        };
        if let Ok(written) = result.as_ref() {
            summary.snapshots += usize::from(*written);
        }
        let error = result.err();
        if let Some(message) = &error {
            failed = true;
            eprintln!("Dependency sync for {id} ({provider}): {message}");
        }
        if let Err(error) = client
            .status(
                id,
                provider,
                error.as_deref(),
                Some(&json!({"repository":repository})),
            )
            .await
        {
            failed = true;
            eprintln!("Dependency collection status for {id}: {error}");
        }
    }
    failed
}

/// Prefer asynchronous exports; older GitHub deployments may only offer the legacy route.
async fn inventory(
    gh: &Gh,
    repository: &str,
    pending: &mut PendingExports,
) -> Result<DependencyInventory, String> {
    inventory_with_timeout(gh, repository, pending, Duration::from_mins(1)).await
}

/// A collection deadline bounds work without losing an asynchronous report identity.
async fn inventory_with_timeout(
    gh: &Gh,
    repository: &str,
    pending: &mut PendingExports,
    timeout: Duration,
) -> Result<DependencyInventory, String> {
    tokio::time::timeout(timeout, async {
        let path = if let Some(path) = pending.get(repository) {
            path.clone()
        } else {
            let base = format!("repos/{repository}/dependency-graph/sbom");
            match gh.get(&format!("{base}/generate-report")).await {
                Err(error) if error.contains("(HTTP 404)") => {
                    return parse::inventory(&gh.get(&base).await?);
                }
                Err(error) => return Err(error),
                Ok(generated) => {
                    let path = export_path(&generated, repository)?;
                    pending.insert(repository.to_owned(), path.clone());
                    path
                }
            }
        };
        loop {
            match gh.get_pending(&path).await {
                Ok(Some(payload)) => {
                    pending.remove(repository);
                    return parse::inventory(&payload);
                }
                Ok(None) => tokio::time::sleep(Duration::from_secs(1)).await,
                Err(error) => {
                    // Expired exports get a fresh report on the next cycle.
                    if error.contains("(HTTP 404)") {
                        pending.remove(repository);
                    }
                    return Err(error);
                }
            }
        }
    })
    .await
    .map_err(|_| {
        "GitHub dependency inventory export is still pending; retrying the same report next poll"
            .to_owned()
    })?
    .map_err(|error| format!("Dependency inventory unavailable: {error}"))
}

/// Only fetch the repository-scoped report returned by GitHub's generation endpoint.
fn export_path(generated: &Value, repository: &str) -> Result<String, String> {
    let url = url::Url::parse(
        generated["sbom_url"]
            .as_str()
            .ok_or("GitHub returned no SBOM report URL")?,
    )
    .map_err(|_| "GitHub returned an invalid SBOM report URL")?;
    let prefix = format!("/repos/{repository}/dependency-graph/sbom/fetch-report/");
    let path = url.path();
    let matched = path
        .get(..prefix.len())
        .is_some_and(|value| value.eq_ignore_ascii_case(&prefix));
    if !matched {
        return Err("GitHub returned a report for a different repository".into());
    }
    let id = &path[prefix.len()..];
    if url.scheme() != "https"
        || url.host_str() != Some("api.github.com")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || id.len() != 36
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
    {
        return Err("GitHub returned an invalid SBOM report URL".into());
    }
    Ok(format!(
        "repos/{repository}/dependency-graph/sbom/fetch-report/{id}"
    ))
}

/// Pagination must finish before any current-security result is recorded.
async fn security(gh: &Gh, repository: &str) -> Result<DependencySecurity, String> {
    let pages = gh
        .get_pages(&format!(
            "repos/{repository}/dependabot/alerts?state=open&per_page=100"
        ))
        .await
        .map_err(|error| {
            format!(
                "Dependabot alerts unavailable: {error}. Dependabot alerts read access is required"
            )
        })?;
    parse::security(&pages, repository)
}

#[cfg(test)]
mod tests;
