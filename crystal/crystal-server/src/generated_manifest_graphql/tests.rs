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

/// Normalized ingestion and typed reads require a principal and never start commands.
#[tokio::test]
async fn evidence_graphql_boundary_is_typed_and_passive() -> Result<(), Box<dyn std::error::Error>>
{
    use crate::{
        auth::authenticate_request,
        state::{AppState, AuthMode},
    };
    use actix_web::test::TestRequest;
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite.clone());
    let state = AppState {
        sample: None,
        auth_mode: AuthMode::Local,
        clerk_authorizer: None,
        clerk_client: None,
        db_pool: pool.clone(),
        local_capability: None,
        local_port: None,
    };
    let auth = authenticate_request(&TestRequest::default().to_http_request(), &state).await?;
    let context = auth.manifest_request_context()?;
    let schema = Schema::build(QueryRoot, GeneratedManifestMutationRoot, EmptySubscription)
        .data(pool.clone())
        .data(state)
        .finish();
    let config = serde_json::json!({"kind":"git_status","id":"git"});
    let declarations = crystal_core::collectors::declarations(
        &serde_json::json!([{"manifestId":"api","repository":[config]}]),
    )?;
    let revision = &declarations[0].revision;
    let observation = serde_json::json!({"schemaVersion":1,"observationId":"event","manifestId":"api","section":"repository","collectorId":"git","integration":"git_status","workspaceId":"laptop","scope":"worktree","planRevision":"plan","collectorRevision":revision,"runId":"run","attempt":1,"observedAt":"2026-09-17T12:00:00Z","startedAt":"2026-09-17T12:00:00Z","inputFingerprint":"input","result":{"kind":"git","data":{"dirty":true,"changedFiles":3,"ahead":1,"behind":0}}});
    let ingest = || {
        Request::new(
            "mutation Record($observation: JSON!) { recordEvidence(observation: $observation) }",
        )
        .variables(Variables::from_json(
            serde_json::json!({"observation":observation}),
        ))
    };
    assert!(!schema.execute(ingest()).await.errors.is_empty());
    let recorded = schema.execute(ingest().data(context.clone())).await;
    assert!(recorded.errors.is_empty(), "{:?}", recorded.errors);
    assert_eq!(recorded.data.into_json()?["recordEvidence"], true);
    assert_eq!(
        schema
            .execute(ingest().data(context.clone()))
            .await
            .data
            .into_json()?["recordEvidence"],
        false
    );
    let history = || {
        Request::new(
            "{ evidenceHistory(manifestId:\"api\",section:REPOSITORY,collectorId:\"git\",workspaceId:\"laptop\") { observationId recordedAt result { __typename ... on GitResult { dirty changedFiles } } } }",
        )
    };
    let result = schema.execute(history().data(context.clone())).await;
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    let value = result.data.into_json()?;
    assert_eq!(
        value["evidenceHistory"][0]["result"]["__typename"],
        "GitResult"
    );
    assert_eq!(value["evidenceHistory"][0]["result"]["changedFiles"], 3);
    let mut other = context.clone();
    other.clerk_org_id = "other".into();
    assert_eq!(
        schema
            .execute(history().data(other))
            .await
            .data
            .into_json()?["evidenceHistory"],
        serde_json::json!([])
    );
    let input=UpsertGeneratedManifestInput{artifact_kind:ArtifactKind::Value,artifact_key:"api".into(),folder_path:None,file_name:Some("index.scry".into()),scry_identifier:Some("diagram".into()),name:Some("API".into()),git_commit_sha:None,content:serde_json::json!({"files":[{"path":"index.scry","content":"api = Manifest(name='API')"}],"manifests":[{"name":"API","manifestId":"api","line_number":1,"repository":[config],"checks":[{"kind":"ruff","id":"lint"}]}]}).to_string()};
    crystal_core::persistence::persist_generated_manifest(&pool, &input, &context).await?;
    let request=Request::new("{ blocks(scryIdentifier:\"diagram\",workspaceId:\"laptop\") { rawJsonString evidence { collectorId section state latest { result { __typename } } } } }").data(auth).data(context);
    let response = schema.execute(request).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json()?;
    assert_eq!(data["blocks"][0]["evidence"][0]["collectorId"], "git");
    assert_eq!(data["blocks"][0]["evidence"][1]["collectorId"], "lint");
    assert_eq!(data["blocks"][0]["evidence"][1]["state"], "WAITING");
    let raw: serde_json::Value = serde_json::from_str(
        data["blocks"][0]["rawJsonString"]
            .as_str()
            .ok_or("missing raw JSON")?,
    )?;
    assert!(raw["repository"].is_array());
    assert!(raw.get("evidence").is_none());
    for old in [
        "{ reportHistory(manifestId:\"api\") }",
        "{ actionHistory(manifestId:\"api\") }",
        "{ diagramMetrics }",
        "{ manifestQuery(source:{},name:\"q\") }",
    ] {
        assert!(!schema.execute(old).await.errors.is_empty());
    }
    sqlite.close().await;
    Ok(())
}

/// Latest summaries and detail pages share immutable evidence, with full-snapshot filtering.
#[tokio::test]
async fn dependency_details_are_bounded_filtered_and_organization_scoped()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let pool = DatabasePool::Sqlite(sqlite);
    let context = ManifestRequestContext {
        clerk_user_id: "user".into(),
        clerk_org_id: "org".into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:admin".into()),
        clerk_org_permissions: vec![],
    };
    let observation: crystal_core::evidence::EvidenceObservation = serde_json::from_value(
        serde_json::json!({"schemaVersion":1,"observationId":"inventory","manifestId":"api","section":"dependencies","collectorId":"sbom","integration":"syft_inventory","workspaceId":"laptop","scope":"worktree","planRevision":"plan","collectorRevision":"revision","runId":"run","attempt":1,"observedAt":"2026-09-17T12:00:00Z","startedAt":"2026-09-17T12:00:00Z","inputFingerprint":"input","result":{"kind":"inventory","data":{"packages":[{"id":"one","name":"Alpha","version":"1","ecosystem":"npm","paths":["web/package-lock.json"],"licenses":["MIT"]},{"id":"two","name":"Beta","version":"2","ecosystem":"pypi","paths":["uv.lock"],"licenses":[]},{"id":"three","name":"Gamma","version":"3","ecosystem":"npm","paths":["web/package-lock.json"],"licenses":["Apache-2.0"]}],"relationships":[{"from":"one","to":"three"}],"complete":true,"artifactHash":"inventory-hash"}}}),
    )?;
    crystal_core::persistence::record_evidence(&pool, &context, observation).await?;
    let schema = Schema::build(QueryRoot, GeneratedManifestMutationRoot, EmptySubscription)
        .data(pool)
        .finish();
    let query = r#"{ evidenceObservation(observationId:"inventory",workspaceId:"laptop") { result { ... on InventoryResult {totalPackages licensedPackages unknownLicensePackages totalRelationships matchingPackages(filter:"NPM") packages(limit:1,offset:1,filter:"npm") {id name} relationships(limit:0) {from to} } } } }"#;
    let response = schema
        .execute(Request::new(query).data(context.clone()))
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json()?;
    let result = &data["evidenceObservation"]["result"];
    assert_eq!(result["totalPackages"], 3);
    assert_eq!(result["licensedPackages"], 2);
    assert_eq!(result["unknownLicensePackages"], 1);
    assert_eq!(result["matchingPackages"], 2);
    assert_eq!(result["packages"][0]["id"], "three");
    assert_eq!(result["relationships"], serde_json::json!([]));
    let filtered = r#"{ evidenceObservation(observationId:"inventory") { result { ... on InventoryResult { matchingPackages(search:"APACHE") packages(ids:["two"],limit:25) {id} } } } }"#;
    let data = schema
        .execute(Request::new(filtered).data(context.clone()))
        .await
        .data
        .into_json()?;
    assert_eq!(data["evidenceObservation"]["result"]["matchingPackages"], 1);
    assert_eq!(
        data["evidenceObservation"]["result"]["packages"][0]["id"],
        "two"
    );
    let too_large = r#"{ evidenceObservation(observationId:"inventory") { result { ... on InventoryResult { packages(limit:101) {id} } } } }"#;
    assert!(
        !schema
            .execute(Request::new(too_large).data(context.clone()))
            .await
            .errors
            .is_empty()
    );
    let mut other = context;
    other.clerk_org_id = "another-org".into();
    assert!(
        schema
            .execute(Request::new(query).data(other))
            .await
            .data
            .into_json()?["evidenceObservation"]
            .is_null()
    );
    Ok(())
}
