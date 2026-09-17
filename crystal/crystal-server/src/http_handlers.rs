//! HTTP handlers for GraphQL routes and playground.

use crate::auth::{
    authenticate_request, has_requested_clerk_org_id, resolve_manifest_request_context,
    unauthorized_response,
};
use crate::roots::{MutationRoot, QueryRoot, SubscriptionRoot};
use crate::state::AppState;
use crate::static_assets;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use async_graphql::Schema;
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};

/// Handle GraphQL queries and mutations.
pub(crate) async fn graphql_handler(
    request: HttpRequest,
    schema: web::Data<Schema<QueryRoot, MutationRoot, SubscriptionRoot>>,
    app_state: web::Data<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let auth = match authenticate_request(&request, app_state.get_ref()).await {
        Ok(auth) => auth,
        Err(message) => {
            return GraphQLResponse::from(async_graphql::Response::from_errors(vec![
                async_graphql::ServerError::new(message, None),
            ]));
        }
    };

    let inner_request = req.into_inner();
    let requires_manifest_context = request_requires_manifest_context(&inner_request.query);
    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok());
    let expected = {
        let connection = request.connection_info();
        format!("{}://{}", connection.scheme(), connection.host())
    };
    let local_editor_allowed = local_editor_origin_allowed(
        origin,
        &expected,
        &crate::server::allowed_origins_from_env(),
    );
    let mut graphql_request = inner_request
        .data(auth.clone())
        .data(crate::editor::EditorRequestAllowed(local_editor_allowed))
        .data(crate::editor::LocalCapabilityAllowed(
            local_capability_allowed(&request, app_state.get_ref()),
        ));
    match resolve_manifest_request_context(&request, &auth, app_state.get_ref()).await {
        Ok(manifest_request_context) => {
            graphql_request = graphql_request.data(manifest_request_context);
        }
        Err(message)
            if requires_manifest_context
                || has_requested_clerk_org_id(&request)
                || message.contains("pass --clerk-org-id")
                || message.contains("not a member of any organization") =>
        {
            return GraphQLResponse::from(async_graphql::Response::from_errors(vec![
                async_graphql::ServerError::new(message, None),
            ]));
        }
        Err(_) => {}
    }

    schema.execute(graphql_request).await.into()
}

/// Permit local source access only from this UI or explicitly trusted loopback development UIs.
fn local_editor_origin_allowed(origin: Option<&str>, expected: &str, trusted: &[String]) -> bool {
    let loopback = |value: &str| {
        url::Url::parse(value).ok().is_some_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
        })
    };
    origin.is_some_and(|origin| {
        loopback(origin)
            && loopback(expected)
            && (origin == expected || trusted.iter().any(|value| value == origin))
    })
}

/// Return whether a GraphQL document uses fields that require Scryr map tenant context.
fn request_requires_manifest_context(query: &str) -> bool {
    query.contains("recordEvidence")
        || query.contains("recordCollectorStatus")
        || query.contains("evidenceHistory")
        || query.contains("evidenceObservation")
        || query.contains("manifestDocument")
        || query.contains("saveManifestDocument")
        || query.contains("upsertGeneratedManifest")
        || query.contains("scryrMaps")
        || query.contains("blocks")
}

/// Render the GraphQL playground UI.
pub(crate) async fn playground_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

/// Serve the embedded Vite map UI and its static assets.
pub(crate) async fn map_ui_handler(
    request: HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    if !static_assets::has_map_ui() {
        return HttpResponse::ServiceUnavailable()
            .content_type("text/plain; charset=utf-8")
            .body(
                "Scryr map UI assets are not embedded in this binary. Build with `mise run release:build`.",
            );
    }

    let requested_path = request.path().trim_start_matches('/');
    let asset_path = if requested_path.is_empty() {
        "index.html"
    } else {
        requested_path
    };

    if is_safe_asset_path(asset_path)
        && let Some(asset) = static_assets::get(asset_path)
    {
        if asset_path == "index.html" {
            return runtime_index(asset.bytes, &state, &request);
        }
        return HttpResponse::Ok()
            .content_type(asset.content_type)
            .body(asset.bytes);
    }

    static_assets::get("index.html").map_or_else(
        || {
            HttpResponse::ServiceUnavailable()
                .content_type("text/plain; charset=utf-8")
                .body("Scryr map UI index.html is missing from this binary.")
        },
        |asset| runtime_index(asset.bytes, &state, &request),
    )
}

fn runtime_index(bytes: &[u8], state: &AppState, request: &HttpRequest) -> HttpResponse {
    let mut config =
        serde_json::json!({ "authMode": state.auth_mode.as_str(), "graphqlEndpoint": "/graphql" });
    if loopback_initial_ui_allowed(request, state.local_port)
        && let Some(capability) = &state.local_capability
    {
        config["localCapability"] = serde_json::json!(capability);
    }
    let html = String::from_utf8_lossy(bytes).replacen(
        "<head>",
        &format!("<head><script>window.__SCRYR_RUNTIME__={config};</script>"),
        1,
    );
    HttpResponse::Ok()
        .insert_header(("cache-control", "no-store"))
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// Check the raw Host header, never a forwarded/proxy-derived authority.
fn loopback_initial_ui_allowed(request: &HttpRequest, port: Option<u16>) -> bool {
    let Some(port) = port else {
        return false;
    };
    let Some(host) = request.headers().get("host").and_then(|v| v.to_str().ok()) else {
        return false;
    };
    let allowed = [
        format!("localhost:{port}"),
        format!("127.0.0.1:{port}"),
        format!("[::1]:{port}"),
    ];
    if !allowed.iter().any(|v| v == host) {
        return false;
    }
    request
        .headers()
        .get("origin")
        .is_none_or(|v| v.to_str().is_ok_and(|v| v == format!("http://{host}")))
}
fn local_capability_allowed(request: &HttpRequest, state: &AppState) -> bool {
    loopback_initial_ui_allowed(request, state.local_port)
        && request.headers().contains_key("origin")
        && state.local_capability.as_ref().is_some_and(|expected| {
            request
                .headers()
                .get("x-scryr-local-capability")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|actual| actual == expected)
        })
}

/// Return whether a requested path can be resolved inside the embedded asset set.
fn is_safe_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains("..")
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

/// Handle GraphQL websocket subscriptions.
#[allow(clippy::future_not_send)]
pub(crate) async fn graphql_ws(
    schema: web::Data<Schema<QueryRoot, MutationRoot, SubscriptionRoot>>,
    req: HttpRequest,
    app_state: web::Data<AppState>,
    payload: web::Payload,
) -> Result<HttpResponse> {
    let auth = match authenticate_request(&req, app_state.get_ref()).await {
        Ok(auth) => auth,
        Err(message) => return Ok(unauthorized_response(&message)),
    };

    let mut data = async_graphql::Data::default();
    if let Ok(manifest_request_context) =
        resolve_manifest_request_context(&req, &auth, app_state.get_ref()).await
    {
        data.insert(manifest_request_context);
    }
    data.insert(auth);

    GraphQLSubscription::new(schema.get_ref().clone())
        .with_data(data)
        .start(&req, payload)
}

#[cfg(test)]
mod tests {
    use super::is_safe_asset_path;

    #[test]
    fn editor_requires_same_origin_or_explicit_loopback_trust() {
        use super::local_editor_origin_allowed as allowed;
        let server = "http://127.0.0.1:9000";
        assert!(allowed(Some(server), server, &[]));
        assert!(!allowed(None, server, &[]));
        assert!(!allowed(Some("http://localhost:3000"), server, &[]));
        assert!(allowed(
            Some("http://localhost:3000"),
            server,
            &["http://localhost:3000".into()]
        ));
        for origin in [
            "null",
            "https://evil.example",
            "http://localhost.evil.example:3000",
        ] {
            assert!(!allowed(Some(origin), server, &[origin.into()]));
        }
    }

    #[tokio::test]
    async fn capability_is_only_served_and_accepted_on_the_local_origin()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::state::{AppState, AuthMode};
        use actix_web::{body::to_bytes, test::TestRequest};
        use crystal_core::persistence::DatabasePool;
        let state = AppState {
            sample: None,
            auth_mode: AuthMode::Local,
            clerk_authorizer: None,
            clerk_client: None,
            db_pool: DatabasePool::Sqlite(sqlx::SqlitePool::connect("sqlite::memory:").await?),
            local_capability: Some("capability-test-token".into()),
            local_port: Some(8000),
        };
        let local = TestRequest::get()
            .insert_header(("host", "127.0.0.1:8000"))
            .to_http_request();
        let body =
            to_bytes(super::runtime_index(b"<head></head>", &state, &local).into_body()).await?;
        assert!(String::from_utf8_lossy(&body).contains("capability-test-token"));
        assert!(!super::local_capability_allowed(&local, &state));
        for (host, origin) in [
            ("evil.example:8000", "http://evil.example:8000"),
            ("127.0.0.1:8000", "https://evil.example"),
            ("127.0.0.1:9000", "http://127.0.0.1:9000"),
            ("127.0.0.1:8000", "http://localhost:3000"),
        ] {
            let request = TestRequest::get()
                .insert_header(("host", host))
                .insert_header(("origin", origin))
                .insert_header(("x-scryr-local-capability", "capability-test-token"))
                .insert_header(("forwarded", "host=127.0.0.1:8000"))
                .to_http_request();
            assert!(!super::local_capability_allowed(&request, &state));
            let body =
                to_bytes(super::runtime_index(b"<head></head>", &state, &request).into_body())
                    .await?;
            assert!(!String::from_utf8_lossy(&body).contains("capability-test-token"));
        }
        let valid = TestRequest::post()
            .insert_header(("host", "127.0.0.1:8000"))
            .insert_header(("origin", "http://127.0.0.1:8000"))
            .insert_header(("x-scryr-local-capability", "capability-test-token"))
            .to_http_request();
        assert!(super::local_capability_allowed(&valid, &state));
        let missing = TestRequest::post()
            .insert_header(("host", "127.0.0.1:8000"))
            .insert_header(("origin", "http://127.0.0.1:8000"))
            .to_http_request();
        assert!(!super::local_capability_allowed(&missing, &state));
        Ok(())
    }

    #[test]
    fn static_asset_path_validation_rejects_traversal() {
        assert!(is_safe_asset_path("assets/index.js"));
        assert!(is_safe_asset_path("favicon/favicon.svg"));

        assert!(!is_safe_asset_path(""));
        assert!(!is_safe_asset_path("../index.html"));
        assert!(!is_safe_asset_path("assets/../index.html"));
        assert!(!is_safe_asset_path("assets\\index.js"));
        assert!(!is_safe_asset_path("assets//index.js"));
        assert!(!is_safe_asset_path("./index.html"));
    }
}
