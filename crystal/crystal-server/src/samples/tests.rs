//! Exercise the bundled catalog through the real listing and source editor.
use super::*;
use crate::{
    auth::authenticate_request,
    editor::EditorService,
    roots::{MutationRoot, QueryRoot},
};
use actix_web::test::TestRequest;
use async_graphql::{EmptySubscription, Request, Schema};
use crystal_core::persistence::{DatabasePool, documents};
use std::collections::HashSet;

#[tokio::test]
async fn hosted_listing_seeds_editable_sources_and_local_listing_stays_empty()
-> Result<(), Box<dyn std::error::Error>> {
    let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let mut state = AppState {
        sample: None,
        auth_mode: AuthMode::Local,
        clerk_authorizer: None,
        clerk_client: None,
        db_pool: DatabasePool::Sqlite(sqlite.clone()),
    };
    let auth = authenticate_request(&TestRequest::default().to_http_request(), &state).await?;
    let context = auth.manifest_request_context()?;
    let request = || {
        Request::new("{ scryrMaps { scryIdentifier folderPath fileName } }")
            .data(auth.clone())
            .data(context.clone())
    };
    let schema = Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
        .data(state.clone())
        .finish();
    let local = schema.execute(request()).await;
    assert!(local.errors.is_empty(), "{:?}", local.errors);
    assert_eq!(local.data.into_json()?["scryrMaps"], serde_json::json!([]));

    // Authentication has already been resolved; switch the application mode
    // without needing a live Clerk service in this resolver contract test.
    state.auth_mode = AuthMode::Clerk;
    let schema = Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
        .data(state.clone())
        .finish();
    let hosted = schema.execute(request()).await;
    assert!(hosted.errors.is_empty(), "{:?}", hosted.errors);
    let maps = hosted.data.into_json()?["scryrMaps"]
        .as_array()
        .ok_or("missing maps")?
        .clone();
    assert_eq!(maps.len(), catalog()?.len());
    assert!(!maps.is_empty());
    let mut identifiers = HashSet::new();
    let mut entrypoints = HashSet::new();
    for map in maps {
        assert_eq!(map["folderPath"], "samples");
        let identifier = map["scryIdentifier"].as_str().ok_or("missing identifier")?;
        assert!(
            identifiers.insert(identifier.to_string()),
            "duplicate bundled identifier"
        );
        let editor = EditorService { local: None };
        let doc = editor.read(&state.db_pool, &context, identifier).await?;
        assert!(doc.writable);
        assert!(!doc.local);
        assert!(
            doc.files
                .iter()
                .any(|file| file.path == doc.entrypoint && !file.content.is_empty())
        );
        entrypoints.insert(doc.entrypoint);
    }
    let source_count = serde_json::from_str::<BTreeMap<String, serde_json::Value>>(include_str!(
        "../samples.generated.json"
    ))?
    .len();
    assert_eq!(entrypoints.len(), source_count);

    assert_bundled_source_editable(&state, &context).await?;

    let anonymous = schema.execute("{ scryrMaps { id } }").await;
    assert!(!anonymous.errors.is_empty());
    Ok(())
}

async fn assert_bundled_source_editable(
    state: &AppState,
    context: &ManifestRequestContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use the source editor's real revision/save flow with a bundled envelope.
    let editor = EditorService { local: None };
    let doc = editor.read(&state.db_pool, context, "mern_diagram").await?;
    let untouched =
        documents::read_document(&state.db_pool, &context.clerk_org_id, "lamp_diagram").await?;
    let mut envelope: serde_json::Value =
        serde_json::from_str(include_str!("../samples.generated.json"))?;
    let mut envelope = envelope["mern"].take();
    envelope["files"][0]["path"] = serde_json::json!(doc.entrypoint);
    let source = envelope["files"][0]["content"]
        .as_str()
        .ok_or("missing source")?;
    envelope["files"][0]["content"] = serde_json::json!(format!("{source}\n# My editable copy\n"));
    let saved = editor
        .save(
            &state.db_pool,
            context,
            "mern_diagram",
            &doc.revision,
            envelope,
        )
        .await?;
    assert!(saved.files[0].content.contains("# My editable copy"));
    ensure_samples(state, context).await?;
    assert_eq!(
        editor
            .read(&state.db_pool, context, "mern_diagram")
            .await?
            .revision,
        saved.revision
    );
    assert_eq!(
        documents::read_document(&state.db_pool, &context.clerk_org_id, "lamp_diagram")
            .await?
            .revision,
        untouched.revision
    );

    Ok(())
}

#[tokio::test]
async fn first_reader_can_initialize_without_gaining_edit_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
    let state = AppState {
        sample: None,
        auth_mode: AuthMode::Clerk,
        clerk_authorizer: None,
        clerk_client: None,
        db_pool: DatabasePool::Sqlite(pool),
    };
    let context = ManifestRequestContext {
        clerk_user_id: "reader".into(),
        clerk_org_id: "reader_organization".into(),
        clerk_org_slug: None,
        clerk_org_role: Some("org:member".into()),
        clerk_org_permissions: vec![],
    };
    ensure_samples(&state, &context).await?;
    let editor = EditorService { local: None };
    let doc = editor
        .read(&state.db_pool, &context, "mern_diagram")
        .await?;
    assert!(!doc.writable);
    assert!(
        editor
            .save(
                &state.db_pool,
                &context,
                "mern_diagram",
                &doc.revision,
                serde_json::json!({})
            )
            .await
            .is_err()
    );
    Ok(())
}
