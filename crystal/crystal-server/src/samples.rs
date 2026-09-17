//! Bundled starter diagrams copied into each hosted organization's own storage.

use crate::state::{AppState, AuthMode};
use crystal_core::{
    generation::map_artifacts_from_manifest_json,
    manifest::{ArtifactKind, ManifestRequestContext, UpsertGeneratedManifestInput},
    persistence::seed_organization_samples,
};
use std::{collections::BTreeMap, sync::OnceLock};

/// Parse and split the trusted build-time catalog once per server process.
fn catalog() -> Result<&'static [UpsertGeneratedManifestInput], String> {
    static CATALOG: OnceLock<Result<Vec<UpsertGeneratedManifestInput>, String>> = OnceLock::new();
    CATALOG
        .get_or_init(load_catalog)
        .as_ref()
        .map(Vec::as_slice)
        .map_err(Clone::clone)
}

fn load_catalog() -> Result<Vec<UpsertGeneratedManifestInput>, String> {
    let sources: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(include_str!("samples.generated.json")).map_err(|e| e.to_string())?;
    let mut samples = Vec::new();
    for (name, mut envelope) in sources {
        // Editor documents are keyed by folder AND entrypoint. Each sample must
        // keep a distinct source even though they share the "samples" folder.
        let entrypoint = format!("{name}.scry");
        let files = envelope["files"]
            .as_array_mut()
            .ok_or("Bundled sample has no source files")?;
        let source = files
            .iter_mut()
            .find(|file| file["path"] == "index.scry")
            .ok_or("Bundled sample is missing index.scry")?;
        source["path"] = serde_json::json!(entrypoint);
        let json = serde_json::to_string(&envelope).map_err(|e| e.to_string())?;
        let maps = map_artifacts_from_manifest_json(&json, &format!("samples/{name}"))
            .map_err(|e| e.to_string())?;
        if maps.is_empty() {
            return Err(format!("Bundled sample {name} has no diagrams"));
        }
        for map in maps {
            samples.push(UpsertGeneratedManifestInput {
                artifact_kind: ArtifactKind::Value,
                artifact_key: map.artifact_key,
                folder_path: Some("samples".into()),
                file_name: Some(entrypoint.clone()),
                scry_identifier: Some(map.map_metadata.scry_identifier),
                name: Some(map.map_metadata.name),
                git_commit_sha: None,
                content: map.manifest_json,
            });
        }
    }
    Ok(samples)
}

/// Initialize after authentication, before the first hosted diagram read.
pub(crate) async fn ensure_samples(
    state: &AppState,
    context: &ManifestRequestContext,
) -> Result<(), String> {
    if state.auth_mode == AuthMode::Clerk {
        seed_organization_samples(&state.db_pool, context, catalog()?).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
