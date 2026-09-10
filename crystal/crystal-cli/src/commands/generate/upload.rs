//! Generated artifact upload through GraphQL.

use crate::auth;
use crate::graphql_client::execute_upsert_generated_manifest_mutation;
use crystal_core::generation::GeneratedMapArtifact;
use crystal_core::manifest::UpsertGeneratedManifestInput;
use crystal_core::persistence::ArtifactKind;
use std::env;
use std::path::Path;
use url::{Host, Url};

/// Shared key used for the single persisted schema artifact.
const SCHEMA_ARTIFACT_KEY: &str = "";
/// Environment variable shared with `scryr serve` for local server port selection.
const PORT_ENV: &str = "PORT";
/// Default port used by `scryr serve` when no `PORT` is configured.
const DEFAULT_LOCAL_GRAPHQL_PORT: u16 = 8000;

/// Return the default local GraphQL endpoint, honoring the shared local `PORT`.
pub(crate) fn default_local_graphql_url() -> String {
    let configured_port = env::var(PORT_ENV).ok();
    default_local_graphql_url_from_port(configured_port.as_deref())
}

/// Resolve the optional bearer token needed for one upload target.
pub(crate) async fn upload_bearer_token(graphql_url: &str) -> Result<Option<String>, String> {
    if let Ok(token) = env::var("SCRYR_TOKEN") {
        return Ok(Some(token));
    }
    if graphql_endpoint_uses_local_auth(graphql_url) {
        return Ok(None);
    }

    auth::access_token().await.map(Some).map_err(|error| {
        format!("{error}\nRun `scryr auth login` before uploading to a hosted Scryr endpoint.")
    })
}

/// Persist the schema and value artifacts produced by one generate run.
pub(crate) async fn persist_generated_artifacts(
    context: GeneratedArtifactPersistence<'_>,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    execute_upsert_generated_manifest_mutation(
        &client,
        context.graphql_url,
        context.bearer_token,
        context.clerk_org_id,
        UpsertGeneratedManifestInput {
            artifact_kind: ArtifactKind::Schema,
            artifact_key: SCHEMA_ARTIFACT_KEY.to_string(),
            folder_path: Some(context.manifest_location.folder_path.clone()),
            file_name: Some(context.manifest_location.file_name.clone()),
            scry_identifier: None,
            name: None,
            git_commit_sha: context.git_commit_sha.clone(),
            content: context.pydantic_schema,
        },
    )
    .await?;
    for artifact in context.map_artifacts {
        execute_upsert_generated_manifest_mutation(
            &client,
            context.graphql_url,
            context.bearer_token,
            context.clerk_org_id,
            UpsertGeneratedManifestInput {
                artifact_kind: ArtifactKind::Value,
                artifact_key: artifact.artifact_key,
                folder_path: Some(context.manifest_location.folder_path.clone()),
                file_name: Some(context.manifest_location.file_name.clone()),
                scry_identifier: Some(artifact.map_metadata.scry_identifier),
                name: Some(artifact.map_metadata.name),
                git_commit_sha: context.git_commit_sha.clone(),
                content: artifact.manifest_json,
            },
        )
        .await?;
    }

    Ok(())
}

/// Values needed to persist generated artifacts through GraphQL.
pub(crate) struct GeneratedArtifactPersistence<'a> {
    /// GraphQL endpoint to receive generated artifacts.
    pub(crate) graphql_url: &'a str,
    /// Optional bearer token used for hosted GraphQL mutations.
    pub(crate) bearer_token: Option<&'a str>,
    /// Optional Clerk organization id to request for server-verified upload scope.
    pub(crate) clerk_org_id: Option<&'a str>,
    /// Generated schema artifact content.
    pub(crate) pydantic_schema: String,
    /// Generated map value artifacts.
    pub(crate) map_artifacts: Vec<GeneratedMapArtifact>,
    /// Normalized source file location metadata.
    pub(crate) manifest_location: ManifestLocation,
    /// Optional git commit SHA.
    pub(crate) git_commit_sha: Option<String>,
}

/// Normalized source location metadata for the uploaded manifest file.
pub(crate) struct ManifestLocation {
    /// Folder path relative to the manifest project directory.
    folder_path: String,
    /// Manifest source file name.
    file_name: String,
}

impl ManifestLocation {
    /// Derive a source location from resolved manifest paths.
    pub(crate) fn from_paths(manifest_dir: &Path, manifest_file: &Path) -> Self {
        let relative = manifest_file
            .strip_prefix(manifest_dir)
            .unwrap_or(manifest_file);
        let folder_path = relative
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let file_name = relative
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        Self {
            folder_path,
            file_name,
        }
    }
}

/// Return the local GraphQL endpoint for one optional port value.
fn default_local_graphql_url_from_port(configured_port: Option<&str>) -> String {
    let port = configured_port
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_LOCAL_GRAPHQL_PORT);
    format!("http://127.0.0.1:{port}/graphql")
}

/// Return whether a GraphQL URL points at the default no-cloud local auth surface.
fn graphql_endpoint_uses_local_auth(graphql_url: &str) -> bool {
    let Ok(url) = Url::parse(graphql_url) else {
        return false;
    };

    match url.host() {
        Some(Host::Domain("localhost")) => true,
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{default_local_graphql_url_from_port, graphql_endpoint_uses_local_auth};

    #[test]
    fn local_graphql_endpoints_use_tokenless_local_auth() {
        assert_eq!(
            default_local_graphql_url_from_port(None),
            "http://127.0.0.1:8000/graphql"
        );
        assert!(graphql_endpoint_uses_local_auth(
            &default_local_graphql_url_from_port(None)
        ));
        assert!(graphql_endpoint_uses_local_auth(
            "http://localhost:8000/graphql"
        ));
        assert!(graphql_endpoint_uses_local_auth(
            "http://[::1]:8000/graphql"
        ));

        assert!(!graphql_endpoint_uses_local_auth(
            "https://graphql.scryr.app/graphql"
        ));
        assert!(!graphql_endpoint_uses_local_auth("/graphql"));
    }

    #[test]
    fn default_local_graphql_url_honors_valid_port_env() {
        assert_eq!(
            default_local_graphql_url_from_port(Some("18080")),
            "http://127.0.0.1:18080/graphql"
        );
    }
}
