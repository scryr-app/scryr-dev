//! Compatibility tests for shared upload models and the GraphQL boundary.

use super::GeneratedManifestMutationRoot;
use crate::roots::QueryRoot;
use async_graphql::{EmptySubscription, Request, Schema, Variables};
use crystal_core::manifest::{
    ManifestRequestContext, UpsertGeneratedManifestInput, UpsertGeneratedManifestPayload,
};
use crystal_core::persistence::{ArtifactKind, DatabasePool};

/// Execute shared client input against the real resolver and verify authorization.
#[tokio::test]
async fn shared_upload_input_preserves_graphql_and_storage_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let schema = Schema::build(QueryRoot, GeneratedManifestMutationRoot, EmptySubscription)
        .data(DatabasePool::Sqlite(sqlite.clone()))
        .finish();
    let input = UpsertGeneratedManifestInput {
        artifact_kind: ArtifactKind::Schema,
        artifact_key: "contract-test".into(),
        folder_path: Some("services/catalog".into()),
        file_name: Some("index.scry".into()),
        scry_identifier: Some("catalog".into()),
        name: Some("Catalog".into()),
        git_commit_sha: Some("abc123".into()),
        content: "{}".into(),
    };
    let request = || {
        Request::new(
        "mutation Upload($input: UpsertGeneratedManifestInput!) { upsertGeneratedManifest(input: $input) { id } }",
    ).variables(Variables::from_json(serde_json::json!({"input": input})))
    };
    let response = schema.execute(request()).await;
    assert_eq!(response.errors.len(), 1);
    assert_eq!(
        response.errors[0].message,
        "request is missing active organization"
    );

    let mut principal = ManifestRequestContext {
        clerk_user_id: "user".into(),
        clerk_org_id: "org".into(),
        clerk_org_slug: Some("example".into()),
        clerk_org_role: None,
        clerk_org_permissions: Vec::new(),
    };
    let denied = schema.execute(request().data(principal.clone())).await;
    assert_eq!(denied.errors.len(), 1);
    assert_eq!(
        denied.errors[0].message,
        "active organization role cannot upload generated manifests"
    );

    principal.clerk_org_role = Some("org:admin".into());
    let response = schema.execute(request().data(principal)).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json()?;
    let id = data["upsertGeneratedManifest"]["id"]
        .as_str()
        .ok_or("missing upload id")?;
    let payload: UpsertGeneratedManifestPayload =
        serde_json::from_value(data["upsertGeneratedManifest"].clone())?;
    assert_eq!(payload.id.to_string(), id);
    let row: (String, String, String, String, String, String, String, String) = sqlx::query_as(
        "SELECT artifact_kind, clerk_org_id, folder_path, file_name, scry_identifier, name, git_commit_sha, content FROM generated_manifests WHERE id = ?",
    ).bind(id).fetch_one(&sqlite).await?;
    assert_eq!(
        row,
        (
            "schema".into(),
            "org".into(),
            "services/catalog".into(),
            "index.scry".into(),
            "catalog".into(),
            "Catalog".into(),
            "abc123".into(),
            "{}".into()
        )
    );
    sqlite.close().await;
    Ok(())
}
