//! Strict conversion of complete GitHub responses to canonical dependency snapshots.
use crystal_core::github_dependencies::{
    DependencyInventory, DependencyPackage, DependencySecurity, GithubDependencyAlert,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Required provider text must not be silently converted to missing/empty data.
fn required(value: &Value, key: &str) -> Result<String, String> {
    value[key]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("GitHub dependency response is missing {key}"))
}

/// Optional absent strings are distinct from malformed provider values.
fn optional(value: &Value, key: &str) -> Result<Option<String>, String> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text.clone())),
        _ => Err(format!("GitHub dependency response has invalid {key}")),
    }
}

/// Count unique package versions, excluding the SBOM's repository/root packages.
pub(super) fn inventory(payload: &Value) -> Result<DependencyInventory, String> {
    let sbom = payload.get("sbom").unwrap_or(payload);
    if !sbom["spdxVersion"]
        .as_str()
        .is_some_and(|version| matches!(version, "SPDX-2.2" | "SPDX-2.3"))
    {
        return Err("GitHub returned an unsupported SPDX dependency inventory".into());
    }
    let packages = sbom["packages"]
        .as_array()
        .ok_or("GitHub returned no dependency package inventory")?;
    if packages.len() > 10_001 {
        return Err("Dependency inventory exceeds 10,000 packages".into());
    }
    let relationships = sbom["relationships"].as_array();
    let mut roots = BTreeSet::new();
    if let Some(describes) = sbom["documentDescribes"].as_array() {
        roots.extend(
            describes
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned),
        );
    }
    if let Some(relationships) = relationships {
        for relation in relationships {
            match relation["relationshipType"].as_str() {
                Some("DESCRIBES") => {
                    roots.insert(required(relation, "relatedSpdxElement")?);
                }
                Some("DESCRIBED_BY") => {
                    roots.insert(required(relation, "spdxElementId")?);
                }
                _ => {}
            }
        }
    }
    // Legacy GitHub exports use this canonical root even when DESCRIBES is missing.
    if packages
        .iter()
        .any(|package| package["SPDXID"] == "SPDXRef-Repository")
    {
        roots.insert("SPDXRef-Repository".into());
    }
    let mut aliases = BTreeMap::new();
    let mut result = BTreeMap::new();
    for package in packages {
        let spdx = required(package, "SPDXID")?;
        if aliases.contains_key(&spdx) {
            return Err("Duplicate SPDX package identity".into());
        }
        if roots.contains(&spdx) {
            aliases.insert(spdx.clone(), spdx);
            continue;
        }
        let name = required(package, "name")?;
        let version = optional(package, "versionInfo")?;
        let known_license = |license: &String| !matches!(license.as_str(), "NOASSERTION" | "NONE");
        let license = optional(package, "licenseConcluded")?
            .filter(known_license)
            .or(optional(package, "licenseDeclared")?.filter(known_license));
        let purl = package["externalRefs"]
            .as_array()
            .and_then(|refs| {
                refs.iter()
                    .find(|reference| reference["referenceType"] == "purl")
            })
            .map(|reference| required(reference, "referenceLocator"))
            .transpose()?;
        if purl
            .as_ref()
            .is_some_and(|value| !value.starts_with("pkg:"))
        {
            return Err("Invalid package URL in dependency inventory".into());
        }
        let id = purl.clone().unwrap_or_else(|| spdx.clone());
        aliases.insert(spdx, id.clone());
        if let Some(saved) = result.get(&id) {
            let saved: &DependencyPackage = saved;
            if saved.name != name || saved.version != version {
                return Err("Conflicting package identities in dependency inventory".into());
            }
        } else {
            result.insert(
                id.clone(),
                DependencyPackage {
                    id,
                    name,
                    version,
                    license,
                    purl,
                },
            );
        }
    }
    let (direct_deps, transitive_deps) = counts(relationships, &roots, &aliases, &result);
    Ok(DependencyInventory {
        observed_at: std::time::SystemTime::now().into(),
        packages: result.into_values().collect(),
        direct_deps,
        transitive_deps,
    })
}

/// Classify direct/transitive packages only when supplied relationships cover every package.
fn counts(
    relationships: Option<&Vec<Value>>,
    roots: &BTreeSet<String>,
    aliases: &BTreeMap<String, String>,
    packages: &BTreeMap<String, DependencyPackage>,
) -> (Option<u64>, Option<u64>) {
    let Some(relationships) = relationships else {
        return (None, None);
    };
    if roots.is_empty() {
        return (None, None);
    }
    let mut edges: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for relation in relationships {
        let Some(mut from) = relation["spdxElementId"].as_str() else {
            return (None, None);
        };
        let Some(mut to) = relation["relatedSpdxElement"].as_str() else {
            return (None, None);
        };
        match relation["relationshipType"].as_str() {
            Some("DEPENDS_ON") => {}
            Some("DEPENDENCY_OF") => {
                std::mem::swap(&mut from, &mut to);
            }
            _ => continue,
        }
        if !aliases.contains_key(from) || !aliases.contains_key(to) {
            return (None, None);
        }
        edges.entry(from).or_default().insert(to);
    }
    let mut direct = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut pending: Vec<&str> = roots.iter().map(String::as_str).collect();
    while let Some(from) = pending.pop() {
        if !visited.insert(from) {
            continue;
        }
        if let Some(targets) = edges.get(from) {
            for to in targets {
                if roots.contains(from) {
                    direct.insert(*to);
                }
                pending.push(to);
            }
        }
    }
    let normalize = |ids: BTreeSet<&str>| -> BTreeSet<String> {
        ids.into_iter()
            .filter_map(|id| aliases.get(id))
            .filter(|id| packages.contains_key(*id))
            .cloned()
            .collect()
    };
    let direct = normalize(direct);
    let reachable = normalize(visited);
    if reachable.len() != packages.len() {
        return (None, None);
    }
    (
        Some(direct.len() as u64),
        Some((packages.len() - direct.len()) as u64),
    )
}

/// Only complete, valid open-alert pages establish a known security state.
pub(super) fn security(payload: &Value, repository: &str) -> Result<DependencySecurity, String> {
    let pages = payload
        .as_array()
        .filter(|pages| !pages.is_empty() && pages.len() <= 100)
        .ok_or("Dependabot returned incomplete or oversized pagination")?;
    let mut alerts = BTreeMap::new();
    for page in pages {
        let values = page
            .as_array()
            .ok_or("Dependabot returned an invalid alert page")?;
        for alert in values {
            let number = alert["number"]
                .as_u64()
                .filter(|number| *number > 0)
                .ok_or("Dependabot returned an invalid alert number")?;
            if alerts.contains_key(&number) || alerts.len() >= 10_000 {
                return Err(
                    "Dependabot returned duplicate alerts or exceeded 10,000 alerts".into(),
                );
            }
            if alert["state"] != "open" {
                return Err("Dependabot returned an unexpected alert state".into());
            }
            let dependency = &alert["dependency"];
            let vulnerability = &alert["security_vulnerability"];
            let advisory = &alert["security_advisory"];
            let severity = required(vulnerability, "severity")?;
            let severity = if severity == "moderate" {
                "medium".to_owned()
            } else {
                severity
            };
            if !matches!(severity.as_str(), "low" | "medium" | "high" | "critical") {
                return Err("Dependabot returned an unknown severity".into());
            }
            let url = required(alert, "html_url")?;
            if !url.eq_ignore_ascii_case(&format!(
                "https://github.com/{repository}/security/dependabot/{number}"
            )) {
                return Err("Dependabot returned an alert for another repository".into());
            }
            alerts.insert(
                number,
                GithubDependencyAlert {
                    number,
                    package: required(&dependency["package"], "name")?,
                    ecosystem: required(&dependency["package"], "ecosystem")?,
                    manifest_path: required(dependency, "manifest_path")?,
                    severity,
                    url,
                    ghsa_id: optional(advisory, "ghsa_id")?,
                    cve_id: optional(advisory, "cve_id")?,
                    summary: optional(advisory, "summary")?,
                    vulnerable_version_range: optional(vulnerability, "vulnerable_version_range")?,
                    first_patched_version: optional(
                        &vulnerability["first_patched_version"],
                        "identifier",
                    )?,
                },
            );
        }
    }
    Ok(DependencySecurity {
        observed_at: std::time::SystemTime::now().into(),
        alerts: alerts.into_values().collect(),
    })
}
