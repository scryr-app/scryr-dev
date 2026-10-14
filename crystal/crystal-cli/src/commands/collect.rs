//! Explicit local collector execution and plan loading.

use crate::args::{CollectArgs, GenerateCommonArgs};
use crystal_core::collection::{CollectionPlan, CollectionRuntime};
use std::path::PathBuf;

/// Load definitions without running any task commands.
pub(super) async fn prepare(
    common: GenerateCommonArgs,
    format: bool,
    check: bool,
) -> Result<Option<CollectionRuntime>, String> {
    let (root, plan) = tokio::task::spawn_blocking(move || {
        let project = super::workflow::Project::new(common)?;
        if format {
            project.tool("--format")?;
        }
        let json = if check {
            project.check()?.json
        } else {
            project.json()?
        };
        let plan = plan_from_json(&json)?;
        let root = project
            .file
            .parent()
            .ok_or("Manifest has no parent directory")?
            .to_path_buf();
        Ok::<(PathBuf, Option<CollectionPlan>), String>((root, plan))
    })
    .await
    .map_err(|error| error.to_string())??;
    match plan {
        Some(plan) => CollectionRuntime::open(root, plan).await.map(Some),
        None => Ok(None),
    }
}

/// Read the versioned plan from the existing manifest envelope.
fn plan_from_json(json: &str) -> Result<Option<CollectionPlan>, String> {
    let envelope: crystal_core::generated_manifest_envelope::GeneratedManifestEnvelope =
        serde_json::from_str(json).map_err(|error| error.to_string())?;
    envelope
        .extra
        .get("collection_plan")
        .map(|value| {
            let plan: CollectionPlan =
                serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
            plan.validate()?;
            Ok((!plan.collectors.is_empty()).then_some(plan))
        })
        .transpose()
        .map(Option::flatten)
}

/// Execute an explicitly selected collector or all declared collectors once.
pub(super) async fn run(args: CollectArgs) -> Result<(), String> {
    let runtime = prepare(args.common, false, false)
        .await?
        .ok_or("This manifest defines no collectors")?;
    let result = run_cancellable(&runtime, args.collector.as_deref()).await;
    // Even failed runs have persisted evidence available to the read-only UI.
    println!(
        "{}",
        serde_json::to_string_pretty(&runtime.snapshot().await?)
            .map_err(|error| error.to_string())?
    );
    result
}

/// Catch terminal signals so dropping the execution future cleans up process groups.
#[cfg(unix)]
async fn run_cancellable(
    runtime: &CollectionRuntime,
    collector: Option<&str>,
) -> Result<(), String> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut interrupt = signal(SignalKind::interrupt()).map_err(|error| error.to_string())?;
    let mut terminate = signal(SignalKind::terminate()).map_err(|error| error.to_string())?;
    tokio::select! {
        result = runtime.run_once(collector) => result,
        _ = interrupt.recv() => Err("Collection interrupted".into()),
        _ = terminate.recv() => Err("Collection terminated".into()),
    }
}

/// Unsupported platforms cannot promise descendant process cleanup.
#[cfg(not(unix))]
async fn run_cancellable(
    _runtime: &CollectionRuntime,
    _collector: Option<&str>,
) -> Result<(), String> {
    Err("Local collectors currently require a Unix platform".into())
}

#[cfg(test)]
mod tests {
    use super::plan_from_json;

    #[test]
    fn legacy_envelopes_have_no_executable_plan() -> Result<(), String> {
        assert!(plan_from_json(r#"{"manifests":[]}"#)?.is_none());
        assert!(
            plan_from_json(
                r#"{"collection_plan":{"version":999,"tasks":[],"collectors":[],"cards":[]}}"#
            )
            .is_err()
        );
        Ok(())
    }
}
