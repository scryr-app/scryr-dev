//! Normalize provider responses into the shared delivery contract.
#![allow(clippy::missing_docs_in_private_items)]
use super::gh::{self, GitHub};
use crystal_core::delivery::{
    ActionsReportSource, DeliveryArtifact, DeliveryBuild, DeliveryCompatibilityEvidence,
    DeliveryDeployment, DeliveryEvidenceState, DeliveryLink, DeliveryRelease,
    DeliveryRuntimeEvidence, DeliverySnapshot, DeliverySourceCategory, DeliverySourceState,
    GitHubDelivery,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::fmt::Write as _;

fn decode<T: DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|e| format!("Malformed GitHub delivery data: {e}"))
}
fn id(value: &Value) -> Result<String, String> {
    value
        .as_u64()
        .filter(|v| *v > 0)
        .map(|v| v.to_string())
        .ok_or_else(|| "Missing GitHub numeric identity".into())
}
fn state(value: &Value, known: &[&str]) -> Value {
    value
        .as_str()
        .filter(|v| known.contains(v))
        .map_or_else(|| json!("unknown"), |v| json!(v))
}
fn conclusion(value: &Value) -> Value {
    if value.is_null() {
        Value::Null
    } else {
        state(
            value,
            &[
                "success",
                "failure",
                "cancelled",
                "skipped",
                "timed_out",
                "action_required",
                "neutral",
                "stale",
                "startup_failure",
            ],
        )
    }
}
fn run_status(value: &Value) -> Value {
    state(
        value,
        &[
            "queued",
            "in_progress",
            "completed",
            "waiting",
            "requested",
            "pending",
        ],
    )
}
fn deployment_status(value: &Value) -> Value {
    state(
        value,
        &[
            "pending",
            "queued",
            "in_progress",
            "success",
            "failure",
            "error",
            "inactive",
        ],
    )
}
fn workflow_id(path: &str) -> String {
    gh::segment(path.strip_prefix(".github/workflows/").unwrap_or(path))
}
fn source(
    snapshot: &mut DeliverySnapshot,
    category: DeliverySourceCategory,
    environment: Option<&str>,
    available: bool,
) {
    snapshot.sources.push(DeliverySourceState {
        category,
        environment: environment.map(str::to_owned),
        state: if available {
            DeliveryEvidenceState::Available
        } else {
            DeliveryEvidenceState::Missing
        },
        message: None,
    });
}

pub(super) fn collect(
    config: &GitHubDelivery,
    manifest_id: &str,
    limit: u32,
    verify: bool,
) -> Result<DeliverySnapshot, String> {
    collect_with_api(
        config,
        manifest_id,
        limit,
        verify,
        &GitHub::new(config.repository.host.clone()),
    )
}

fn collect_with_api(
    config: &GitHubDelivery,
    manifest_id: &str,
    limit: u32,
    verify: bool,
    api: &GitHub,
) -> Result<DeliverySnapshot, String> {
    let repository = format!("{}/{}", config.repository.owner, config.repository.name);
    let base = format!("repos/{repository}");
    let mut snapshot = DeliverySnapshot::new(config.repository.host.clone(), repository.clone());
    snapshot.configuration = Some(config.clone());
    collect_builds(&mut snapshot, api, &base, config, limit)?;
    let has_builds = !snapshot.builds.is_empty();
    source(
        &mut snapshot,
        DeliverySourceCategory::Builds,
        None,
        has_builds,
    );
    collect_releases(&mut snapshot, api, &base, config, limit, verify)?;
    let has_releases = !snapshot.releases.is_empty();
    let has_artifacts = !snapshot.artifacts.is_empty();
    source(
        &mut snapshot,
        DeliverySourceCategory::Releases,
        None,
        has_releases,
    );
    source(
        &mut snapshot,
        DeliverySourceCategory::Artifacts,
        None,
        has_artifacts,
    );
    let has_provenance = snapshot.artifacts.iter().any(|artifact| {
        artifact.provenance == crystal_core::delivery::DeliveryVerification::Verified
    });
    source(
        &mut snapshot,
        DeliverySourceCategory::Provenance,
        None,
        has_provenance,
    );
    collect_environments(
        &mut snapshot,
        api,
        &base,
        &repository,
        config,
        limit,
        manifest_id,
    )?;
    snapshot.validate_for_manifest(manifest_id)?;
    Ok(snapshot)
}

fn collect_builds(
    snapshot: &mut DeliverySnapshot,
    api: &GitHub,
    base: &str,
    config: &GitHubDelivery,
    limit: u32,
) -> Result<(), String> {
    let runs = api.list(
        &format!(
            "{base}/actions/workflows/{}/runs",
            workflow_id(&config.builds.workflow)
        ),
        Some("workflow_runs"),
        limit,
    )?;
    for run in runs {
        let run_id = id(&run["id"])?;
        let attempt = run["run_attempt"]
            .as_u64()
            .ok_or("Build run is missing its attempt")?;
        let jobs = api.list(
            &format!("{base}/actions/runs/{run_id}/attempts/{attempt}/jobs"),
            Some("jobs"),
            100,
        )?;
        snapshot.builds.push(build(&run, &jobs)?);
        for artifact in api.list(
            &format!("{base}/actions/runs/{run_id}/artifacts"),
            Some("artifacts"),
            100,
        )? {
            let name = artifact["name"]
                .as_str()
                .ok_or("Artifact is missing its name")?;
            if config.builds.artifact_names.is_empty()
                || config.builds.artifact_names.iter().any(|v| v == name)
            {
                snapshot
                    .artifacts
                    .push(actions_artifact(&artifact, &run_id)?);
            }
        }
    }
    Ok(())
}

fn collect_releases(
    snapshot: &mut DeliverySnapshot,
    api: &GitHub,
    base: &str,
    config: &GitHubDelivery,
    limit: u32,
    verify: bool,
) -> Result<(), String> {
    for release in api.list(&format!("{base}/releases"), None, limit)? {
        if (!config.releases.include_drafts && release["draft"] == true)
            || (!config.releases.include_prereleases && release["prerelease"] == true)
        {
            continue;
        }
        let release_id = id(&release["id"])?;
        let assets = api.list(&format!("{base}/releases/{release_id}/assets"), None, 100)?;
        let mut asset_ids = Vec::new();
        for asset in assets {
            let mut artifact = release_asset(&asset, &release_id)?;
            if verify {
                artifact.provenance = verify_asset(
                    api,
                    base,
                    &snapshot.repository,
                    &config.builds.workflow,
                    &asset,
                    &mut artifact.digest,
                )?;
            }
            asset_ids.push(artifact.id.clone());
            snapshot.artifacts.push(artifact);
        }
        snapshot.releases.push(decode::<DeliveryRelease>(json!({
            "id":release_id,"tag":release["tag_name"],"name":release["name"],
            "publishedAt":release["published_at"],"draft":release["draft"],
            "prerelease":release["prerelease"],"url":release["html_url"],"assetIds":asset_ids,
        }))?);
    }
    Ok(())
}

fn collect_environments(
    snapshot: &mut DeliverySnapshot,
    api: &GitHub,
    base: &str,
    repository: &str,
    config: &GitHubDelivery,
    limit: u32,
    manifest_id: &str,
) -> Result<(), String> {
    for environment in &config.environments {
        let deployments = api.list(
            &format!(
                "{base}/deployments?environment={}&task={}",
                gh::segment(&environment.deployments.environment),
                gh::segment(&environment.deployments.task)
            ),
            None,
            limit,
        )?;
        source(
            snapshot,
            DeliverySourceCategory::Deployments,
            Some(&environment.name),
            !deployments.is_empty(),
        );
        for deployment in deployments {
            let deployment_id = id(&deployment["id"])?;
            if deployment["environment"].as_str()
                != Some(environment.deployments.environment.as_str())
            {
                return Err("GitHub returned a deployment for another environment".into());
            }
            let statuses = api.list(
                &format!("{base}/deployments/{deployment_id}/statuses"),
                None,
                100,
            )?;
            snapshot.deployments.push(deployment_record(
                &deployment,
                &statuses,
                &environment.name,
            )?);
            if let Some(link) =
                deployment_link(&deployment, &deployment_id, manifest_id, &environment.name)?
            {
                snapshot.links.push(link);
            }
        }
        for (report_source, category) in [
            (&environment.runtime, DeliverySourceCategory::Runtime),
            (
                &environment.compatibility,
                DeliverySourceCategory::Compatibility,
            ),
        ] {
            let result = report_source.as_ref().map_or(Ok(None), |report_source| {
                report(
                    api,
                    base,
                    repository,
                    report_source,
                    limit,
                    manifest_id,
                    &environment.name,
                )
            });
            match category {
                DeliverySourceCategory::Runtime => {
                    if let Some(evidence) = report_result::<DeliveryRuntimeEvidence>(
                        result,
                        snapshot,
                        category,
                        &environment.name,
                    ) {
                        snapshot.runtime.push(evidence);
                    }
                }
                DeliverySourceCategory::Compatibility => {
                    if let Some(evidence) = report_result::<DeliveryCompatibilityEvidence>(
                        result,
                        snapshot,
                        category,
                        &environment.name,
                    ) {
                        snapshot.compatibility.push(evidence);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}

fn deployment_link(
    deployment: &Value,
    deployment_id: &str,
    manifest_id: &str,
    environment: &str,
) -> Result<Option<DeliveryLink>, String> {
    let payload = &deployment["payload"];
    let link_value = if payload["schema"] == "scryr.delivery.v1" {
        payload
    } else {
        &payload["scryr"]
    };
    if link_value.is_null() {
        return Ok(None);
    }
    let mut normalized = link_value.clone();
    if normalized
        .get("deploymentId")
        .is_some_and(|value| !value.is_null() && value.as_str() != Some(deployment_id))
    {
        return Err("Deployment payload references another deployment".into());
    }
    normalized
        .as_object_mut()
        .ok_or("Delivery payload must be an object")?
        .insert("deploymentId".into(), json!(deployment_id));
    let link: DeliveryLink = decode(normalized)?;
    if link.manifest_id != manifest_id
        || link.environment != environment
        || link.deployment_id != deployment_id
    {
        return Err(
            "Deployment delivery link does not match its manifest/environment/deployment".into(),
        );
    }
    Ok(Some(link))
}

fn report_result<T: DeserializeOwned>(
    result: Result<Option<Value>, String>,
    snapshot: &mut DeliverySnapshot,
    category: DeliverySourceCategory,
    environment: &str,
) -> Option<T> {
    if let Ok(evidence) = result.and_then(|value| value.map(decode).transpose()) {
        source(snapshot, category, Some(environment), evidence.is_some());
        evidence
    } else {
        snapshot.sources.push(DeliverySourceState {
            category,
            environment: Some(environment.into()),
            state: DeliveryEvidenceState::Error,
            message: Some("Unable to collect configured workflow evidence; check workflow, artifact and repository read permissions".into()),
        });
        None
    }
}

fn build(run: &Value, jobs: &[Value]) -> Result<DeliveryBuild, String> {
    let run_id = id(&run["id"])?;
    let mut records = Vec::new();
    for job in jobs {
        if id(&job["run_id"])? != run_id || job["run_attempt"] != run["run_attempt"] {
            return Err("Job does not belong to the selected run attempt".into());
        }
        records.push(json!({"id":id(&job["id"])? ,"name":job["name"],"status":run_status(&job["status"]),"conclusion":conclusion(&job["conclusion"]),"url":job["html_url"]}));
    }
    decode(json!({
        "id":run_id,"attempt":run["run_attempt"],"workflow":run["path"],
        "name":run["display_title"].as_str().or_else(|| run["name"].as_str()).unwrap_or("Build"),
        "commitSha":run["head_sha"],"status":run_status(&run["status"]),"conclusion":conclusion(&run["conclusion"]),
        "createdAt":run["created_at"],"updatedAt":run["updated_at"],"url":run["html_url"],"jobs":records,
    }))
}
fn actions_artifact(value: &Value, run_id: &str) -> Result<DeliveryArtifact, String> {
    decode(
        json!({"id":id(&value["id"])? ,"kind":"actions_artifact","name":value["name"],"digest":value["digest"],"runId":run_id,"releaseId":null,"expired":value["expired"],"url":value["url"],"provenance":"unknown"}),
    )
}
fn release_asset(value: &Value, release_id: &str) -> Result<DeliveryArtifact, String> {
    decode(
        json!({"id":id(&value["id"])? ,"kind":"release_asset","name":value["name"],"digest":value["digest"],"runId":null,"releaseId":release_id,"expired":false,"url":value["browser_download_url"],"provenance":"unknown"}),
    )
}
fn deployment_record(
    value: &Value,
    statuses: &[Value],
    environment: &str,
) -> Result<DeliveryDeployment, String> {
    let current = statuses.first().unwrap_or(&Value::Null);
    let events: Vec<_> = statuses.iter().map(|event| json!({"status":deployment_status(&event["state"]),"createdAt":event["created_at"],"url":event["log_url"].as_str().filter(|s| !s.is_empty())})).collect();
    decode(
        json!({"id":id(&value["id"])? ,"environment":environment,"commitSha":value["sha"],"status":deployment_status(&current["state"]),"createdAt":value["created_at"],"updatedAt":current.get("created_at").unwrap_or_else(|| &value["created_at"]),"url":current["environment_url"].as_str().filter(|s| !s.is_empty()).or_else(|| current["log_url"].as_str().filter(|s| !s.is_empty())),"statuses":events}),
    )
}

fn report(
    api: &GitHub,
    base: &str,
    repository: &str,
    source: &ActionsReportSource,
    limit: u32,
    manifest_id: &str,
    environment: &str,
) -> Result<Option<Value>, String> {
    let runs = api.list(
        &format!(
            "{base}/actions/workflows/{}/runs?status=success",
            workflow_id(&source.workflow)
        ),
        Some("workflow_runs"),
        limit,
    )?;
    for run in runs.iter().filter(|run| {
        run["conclusion"] == "success"
            && run["head_repository"]["full_name"].as_str() == Some(repository)
            && !run["event"]
                .as_str()
                .unwrap_or("")
                .starts_with("pull_request")
            && run["path"].as_str() == Some(source.workflow.as_str())
    }) {
        let run_id = id(&run["id"])?;
        let artifacts = api.list(
            &format!(
                "{base}/actions/runs/{run_id}/artifacts?name={}",
                gh::segment(&source.artifact_name)
            ),
            Some("artifacts"),
            100,
        )?;
        let matching: Vec<_> = artifacts
            .iter()
            .filter(|a| a["name"].as_str() == Some(source.artifact_name.as_str()))
            .collect();
        let artifact = match matching.as_slice() {
            [] => continue,
            [artifact] => *artifact,
            _ => return Err("Ambiguous report artifacts in the selected workflow run".into()),
        };
        if artifact["expired"] == true {
            continue;
        }
        if artifact["size_in_bytes"]
            .as_u64()
            .is_none_or(|n| n > 2_000_000)
        {
            return Err("Workflow evidence artifact exceeds 2 MB or has no size".into());
        }
        let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
        let destination = directory
            .path()
            .to_str()
            .ok_or("Invalid temporary report path")?;
        gh::execute_bounded(
            &[
                "run",
                "download",
                &run_id,
                "--repo",
                &format!("{}/{repository}", api.host),
                "--name",
                &source.artifact_name,
                "--dir",
                destination,
            ],
            2_000_000,
            Some(directory.path()),
        )?;
        let path = directory.path().join("report.json");
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| "Workflow evidence artifact must contain report.json at its root")?;
        if !metadata.file_type().is_file() || metadata.len() > 2_000_000 {
            return Err("Workflow report.json must be a regular file no larger than 2 MB".into());
        }
        let evidence: Value =
            serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if evidence["schema"].as_str() != Some(source.schema.as_str()) {
            return Err("Workflow report schema does not match configuration".into());
        }
        if evidence["manifestId"].as_str() == Some(manifest_id)
            && evidence["environment"].as_str() == Some(environment)
        {
            return Ok(Some(evidence));
        }
    }
    Ok(None)
}

fn verify_asset(
    api: &GitHub,
    base: &str,
    repository: &str,
    workflow: &str,
    asset: &Value,
    digest: &mut Option<String>,
) -> Result<crystal_core::delivery::DeliveryVerification, String> {
    use crystal_core::delivery::DeliveryVerification;
    use sha2::{Digest, Sha256};
    if asset["size"].as_u64().is_none_or(|size| size > 256_000_000) {
        return Err("Release asset exceeds the 256 MB attestation verification limit".into());
    }
    let bytes = gh::execute_bounded(
        &[
            "api",
            "--hostname",
            &api.host,
            "--method",
            "GET",
            "--header",
            "Accept: application/octet-stream",
            &format!("{base}/releases/assets/{}", id(&asset["id"])?),
        ],
        256_000_000,
        None,
    )?;
    let computed =
        Sha256::digest(&bytes)
            .iter()
            .fold(String::from("sha256:"), |mut digest, byte| {
                let _ = write!(digest, "{byte:02x}");
                digest
            });
    if digest
        .as_ref()
        .is_some_and(|reported| reported != &computed)
    {
        return Err("Release asset bytes do not match the GitHub digest".into());
    }
    *digest = Some(computed);
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
    let path = directory.path().join("artifact");
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let output = gh::execute(&[
        "attestation",
        "verify",
        path.to_str().ok_or("Invalid temporary asset path")?,
        "--hostname",
        &api.host,
        "--repo",
        repository,
        "--signer-workflow",
        &format!("{}/{repository}/{workflow}", api.host),
        "--format",
        "json",
    ]);
    // A failed invocation might be a missing attestation or transport failure, not
    // a cryptographic rejection. Keep unknown instead of inventing a failed proof.
    let Ok(output) = output else {
        return Ok(DeliveryVerification::Unknown);
    };
    let result: Value =
        serde_json::from_slice(&output).map_err(|_| "Malformed attestation verification output")?;
    if result.as_array().is_some_and(|items| !items.is_empty()) {
        Ok(DeliveryVerification::Verified)
    } else {
        Err("Attestation verification returned no verified attestations".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn future_provider_states_remain_unknown() {
        assert_eq!(run_status(&json!("new_state")), json!("unknown"));
        assert_eq!(conclusion(&Value::Null), Value::Null);
        assert_eq!(deployment_status(&Value::Null), json!("unknown"));
    }
    #[test]
    fn artifact_kind_and_unknown_provenance_are_preserved() -> Result<(), String> {
        let artifact = actions_artifact(
            &json!({"id":1,"name":"archive","expired":true,"digest":null,"url":"https://api.github.com/repos/a/b/actions/artifacts/1"}),
            "9",
        )?;
        assert!(artifact.expired);
        assert_eq!(artifact.run_id.as_deref(), Some("9"));
        assert_eq!(
            artifact.provenance,
            crystal_core::delivery::DeliveryVerification::Unknown
        );
        Ok(())
    }
    #[test]
    fn jobs_from_another_attempt_are_rejected() {
        assert!(
            build(
                &json!({"id":1,"run_attempt":2}),
                &[json!({"run_id":1,"run_attempt":1})]
            )
            .is_err()
        );
    }

    fn native_fixture() -> Result<(GitHubDelivery, std::collections::HashMap<String, Value>), String>
    {
        let sha = "a".repeat(40);
        let digest = format!("sha256:{}", "b".repeat(64));
        let config = decode(
            json!({"kind":"github","repository":{"host":"github.com","owner":"acme","name":"api"},"builds":{"workflow":".github/workflows/build.yml","artifactNames":["binary"]},"releases":{"includeDrafts":false,"includePrereleases":false},"environments":[{"name":"production","deployments":{"environment":"production","task":"deploy"},"runtime":null,"compatibility":null,"rollback":{"requireArtifactAvailable":true,"requireVerifiedProvenance":true,"requireCompatibilityEvidence":true}}]}),
        )?;
        let responses = [
            ("repos/acme/api/actions/workflows/build.yml/runs?per_page=20&page=1", json!({"workflow_runs":[{"id":10,"run_attempt":2,"path":".github/workflows/build.yml","display_title":"Build","head_sha":sha,"status":"completed","conclusion":"success","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:01:00Z","html_url":"https://github.com/acme/api/actions/runs/10"}]})),
            ("repos/acme/api/actions/runs/10/attempts/2/jobs?per_page=100&page=1", json!({"jobs":[{"id":11,"run_id":10,"run_attempt":2,"name":"build","status":"completed","conclusion":"success","html_url":"https://github.com/acme/api/actions/runs/10/jobs/11"}]})),
            ("repos/acme/api/actions/runs/10/artifacts?per_page=100&page=1", json!({"artifacts":[{"id":12,"name":"binary","digest":digest,"expired":true,"url":"https://api.github.com/repos/acme/api/actions/artifacts/12"}]})),
            ("repos/acme/api/releases?per_page=20&page=1", json!([{"id":20,"tag_name":"v1.0.0","name":"Release","published_at":"2026-01-01T00:02:00Z","draft":false,"prerelease":false,"html_url":"https://github.com/acme/api/releases/tag/v1.0.0"}])),
            ("repos/acme/api/releases/20/assets?per_page=100&page=1", json!([{"id":21,"name":"binary","digest":digest,"browser_download_url":"https://github.com/acme/api/releases/download/v1.0.0/binary"}])),
            ("repos/acme/api/deployments?environment=production&task=deploy&per_page=20&page=1", json!([{"id":30,"environment":"production","sha":sha,"created_at":"2026-01-01T00:03:00Z","payload":{"scryr":{"schema":"scryr.delivery.v1","manifestId":"api","environment":"production","buildRunId":"10","buildAttempt":2,"sourceRevision":sha,"artifactDigest":digest,"releaseId":"20","configurationRevision":"config-1"}}}])),
            ("repos/acme/api/deployments/30/statuses?per_page=100&page=1", json!([{"state":"success","created_at":"2026-01-01T00:04:00Z","environment_url":"https://api.example.com","log_url":"https://github.com/acme/api/actions/runs/10"}])),
        ].into_iter().map(|(key, value)| (key.to_owned(), value)).collect();
        Ok((config, responses))
    }

    #[test]
    fn native_collection_links_explicit_identities_without_inventing_runtime() -> Result<(), String>
    {
        let (config, responses) = native_fixture()?;
        let snapshot = collect_with_api(&config, "api", 20, false, &GitHub::fixture(responses))?;
        assert_eq!(snapshot.builds[0].attempt, 2);
        assert_eq!(snapshot.builds[0].jobs.len(), 1);
        assert_eq!(snapshot.artifacts.len(), 2);
        assert!(snapshot.artifacts[0].expired);
        assert_eq!(snapshot.releases[0].asset_ids, ["21"]);
        assert_eq!(snapshot.links[0].deployment_id, "30");
        assert_eq!(snapshot.links[0].build_run_id, "10");
        assert!(snapshot.runtime.is_empty());
        assert!(snapshot.compatibility.is_empty());
        assert!(
            snapshot.artifacts.iter().all(|artifact| artifact.provenance
                == crystal_core::delivery::DeliveryVerification::Unknown)
        );
        Ok(())
    }

    #[test]
    fn malformed_binding_and_provider_failure_do_not_publish_success() -> Result<(), String> {
        let (config, mut responses) = native_fixture()?;
        let endpoint =
            "repos/acme/api/deployments?environment=production&task=deploy&per_page=20&page=1";
        responses.get_mut(endpoint).ok_or("fixture")?[0]["payload"]["scryr"]["deploymentId"] =
            json!("999");
        assert!(
            collect_with_api(
                &config,
                "api",
                20,
                false,
                &GitHub::fixture(responses.clone())
            )
            .is_err()
        );
        responses.remove("repos/acme/api/actions/runs/10/attempts/2/jobs?per_page=100&page=1");
        assert!(collect_with_api(&config, "api", 20, false, &GitHub::fixture(responses)).is_err());
        Ok(())
    }

    #[test]
    fn optional_report_failure_preserves_native_evidence_as_an_explicit_error() -> Result<(), String>
    {
        let (mut config, responses) = native_fixture()?;
        config.environments[0].runtime = Some(decode(
            json!({"workflow":".github/workflows/runtime.yml","artifactName":"runtime","schema":"scryr.runtime.v1","maxAgeSeconds":900}),
        )?);
        let snapshot = collect_with_api(&config, "api", 20, false, &GitHub::fixture(responses))?;
        assert_eq!(snapshot.builds.len(), 1);
        assert_eq!(snapshot.deployments.len(), 1);
        assert!(snapshot.runtime.is_empty());
        assert!(
            snapshot
                .sources
                .iter()
                .any(|source| source.category == DeliverySourceCategory::Runtime
                    && source.state == DeliveryEvidenceState::Error)
        );
        Ok(())
    }

    #[test]
    fn expired_report_artifact_is_missing_not_verification() -> Result<(), String> {
        let (mut config, mut responses) = native_fixture()?;
        config.environments[0].runtime = Some(decode(
            json!({"workflow":".github/workflows/runtime.yml","artifactName":"runtime","schema":"scryr.runtime.v1","maxAgeSeconds":900}),
        )?);
        responses.insert("repos/acme/api/actions/workflows/runtime.yml/runs?status=success&per_page=20&page=1".into(), json!({"workflow_runs":[{"id":40,"conclusion":"success","head_repository":{"full_name":"acme/api"},"event":"workflow_dispatch","path":".github/workflows/runtime.yml"}]}));
        responses.insert(
            "repos/acme/api/actions/runs/40/artifacts?name=runtime&per_page=100&page=1".into(),
            json!({"artifacts":[{"id":41,"name":"runtime","expired":true}]}),
        );
        let snapshot = collect_with_api(&config, "api", 20, false, &GitHub::fixture(responses))?;
        assert!(snapshot.runtime.is_empty());
        assert!(
            snapshot
                .sources
                .iter()
                .any(|source| source.category == DeliverySourceCategory::Runtime
                    && source.state == DeliveryEvidenceState::Missing)
        );
        Ok(())
    }
}
