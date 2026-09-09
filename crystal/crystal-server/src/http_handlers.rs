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
    let mut graphql_request = inner_request.data(auth.clone());
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

/// Return whether a GraphQL document uses fields that require Scryr map tenant context.
fn request_requires_manifest_context(query: &str) -> bool {
    query.contains("recordActionRun")
        || query.contains("actionHistory")
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
pub(crate) async fn map_ui_handler(request: HttpRequest) -> HttpResponse {
    if !static_assets::has_map_ui() {
        return HttpResponse::ServiceUnavailable()
            .content_type("text/plain; charset=utf-8")
            .body(
                "Scryr map UI assets are not embedded in this binary. Build with `mise run build:oss`.",
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
        |asset| {
            HttpResponse::Ok()
                .content_type(asset.content_type)
                .body(asset.bytes)
        },
    )
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
