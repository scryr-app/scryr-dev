//! Manifest generation command workflow.

mod executor;
mod outputs;
mod upload;

use crate::args::GenerateRequest;
use crate::manifest_paths::{derive_manifest_artifact_key, resolve_manifest_file};
use crate::manifest_python::ManifestPythonMode;
use crystal_core::generation::map_artifacts_from_manifest_json;
use executor::prepare_manifest_executor;
use outputs::render_stdout_target;
use upload::{
    GeneratedArtifactPersistence, ManifestLocation, default_local_graphql_url,
    persist_generated_artifacts, upload_bearer_token,
};

/// Execute manifest generation.
pub(super) async fn run(args: &GenerateRequest) -> Result<(), String> {
    let manifest_dir = args.manifest_dir.canonicalize().map_err(|error| {
        format!(
            "Failed to resolve manifest directory {}: {error}",
            args.manifest_dir.display()
        )
    })?;
    let manifest_file = resolve_manifest_file(&manifest_dir, &args.manifest_file)?;
    let artifact_key = derive_manifest_artifact_key(&manifest_dir, &manifest_file);
    let manifest_executor = prepare_manifest_executor(args, &manifest_dir)?;

    if render_stdout_target(
        args.output,
        args,
        &manifest_executor,
        &manifest_dir,
        &manifest_file,
    )? {
        return Ok(());
    }

    println!("Generating manifest artifacts...");

    let pydantic_schema =
        manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Schema)?;
    let manifest_json =
        manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Json)?;
    let map_artifacts = map_artifacts_from_manifest_json(&manifest_json, &artifact_key)?;
    let default_graphql_url = default_local_graphql_url();
    let graphql_url = args
        .graphql_url
        .as_deref()
        .unwrap_or(default_graphql_url.as_str());
    let bearer_token = upload_bearer_token(graphql_url).await?;

    let manifest_location = ManifestLocation::from_paths(&manifest_dir, &manifest_file);
    persist_generated_artifacts(GeneratedArtifactPersistence {
        graphql_url,
        bearer_token: bearer_token.as_deref(),
        pydantic_schema,
        map_artifacts,
        manifest_location,
        clerk_org_id: args.clerk_org_id.as_deref(),
        git_commit_sha: args.git_commit_sha.clone(),
    })
    .await?;
    println!("Persisted generated manifest artifacts through GraphQL");

    Ok(())
}
