//! Manifest generation command workflow.

mod executor;
mod upload;

use crate::args::{GenerateOutput, GenerateRequest};
use crate::manifest_paths::{derive_manifest_artifact_key, resolve_manifest_file};
use crate::manifest_python::ManifestPythonMode;
use crystal_core::generation::map_artifacts_from_manifest_json;
use crystal_core::generation::{render_compose_yaml, render_devcontainer_json, render_mise_toml};
use executor::prepare_manifest_executor;
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

    match args.output {
        GenerateOutput::Types => {
            let types_json =
                manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Types)?;
            print!("{types_json}");
            return Ok(());
        }
        GenerateOutput::Schema => {
            let schema_json =
                manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Schema)?;
            print!("{schema_json}");
            return Ok(());
        }
        GenerateOutput::ArtifactJson => {
            let manifest_json =
                manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Json)?;
            print!("{manifest_json}");
            return Ok(());
        }
        GenerateOutput::Mise => {
            let manifest_json =
                manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Json)?;
            print!(
                "{}",
                render_mise_toml(&manifest_json, args.forge.as_deref())?
            );
            return Ok(());
        }
        GenerateOutput::Compose => {
            let manifest_json =
                manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Json)?;
            print!(
                "{}",
                render_compose_yaml(&manifest_json, args.forge.as_deref())?
            );
            return Ok(());
        }
        GenerateOutput::Devcontainer => {
            let manifest_json =
                manifest_executor.run(&manifest_dir, &manifest_file, ManifestPythonMode::Json)?;
            print!(
                "{}",
                render_devcontainer_json(&manifest_json, args.forge.as_deref())?
            );
            return Ok(());
        }
        GenerateOutput::Upload => {}
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
