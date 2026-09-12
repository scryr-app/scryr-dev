//! Server bootstrap and Actix wiring.

use crate::health::{health_handler, readiness_handler};
use crate::http_handlers::{graphql_handler, graphql_ws, map_ui_handler, playground_handler};
use crate::roots::{MutationRoot, QueryRoot};
use crate::state::{AppState, AuthMode, ServerArgs};
use actix_cors::Cors;
use actix_web::{App, HttpServer, guard, web};
use async_graphql::{EmptySubscription, Schema};
use clerk_rs::{
    ClerkConfiguration, clerk::Clerk, validators::authorizer::ClerkAuthorizer,
    validators::jwks::MemoryCacheJwksProvider,
};
use crystal_core::persistence;
use std::env;

/// Build and run the GraphQL HTTP server.
///
/// # Errors
///
/// Returns an error if storage initialization fails, server configuration is
/// invalid, the bind address cannot be opened, or the HTTP server exits with an
/// error.
pub async fn run(args: ServerArgs) -> std::io::Result<()> {
    start(args).await?.await
}

/// Initialize storage and bind the HTTP listener before returning the server.
///
/// # Errors
/// Returns initialization or bind errors without starting background work.
pub async fn start(args: ServerArgs) -> std::io::Result<actix_web::dev::Server> {
    start_with_workspace(args, None).await
}

/// Start a server with an optional explicitly selected local editor workspace.
///
/// # Errors
/// Returns configuration, storage, or listener errors.
pub async fn start_with_workspace(
    args: ServerArgs,
    workspace: Option<crate::editor::LocalWorkspace>,
) -> std::io::Result<actix_web::dev::Server> {
    if workspace.is_some() && !matches!(args.host.as_str(), "127.0.0.1" | "localhost" | "::1") {
        return Err(std::io::Error::other(
            "Local source editing requires a loopback host; use --server-only for network hosting",
        ));
    }
    let db_pool = persistence::connect_from_env()
        .await
        .map_err(std::io::Error::other)?;

    let host = args.host;
    let port = args.port;
    let auth_mode = AuthMode::resolve(args.auth_mode, &host);

    let clerk_client = env::var("CLERK_SECRET_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|secret_key| Clerk::new(ClerkConfiguration::new(secret_key)));
    let clerk_authorizer = clerk_client
        .clone()
        .map(|clerk| ClerkAuthorizer::new(MemoryCacheJwksProvider::new(clerk), true));

    let app_state = AppState {
        sample: args.sample,
        auth_mode,
        clerk_authorizer,
        clerk_client,
        db_pool,
    };

    let metrics = crate::runtime_metrics::RuntimeMetrics::from_workspace(workspace.as_ref())?;
    let schema = Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
        .data(crate::editor::EditorService { local: workspace })
        .data(app_state.clone())
        .data(app_state.db_pool.clone())
        .data(metrics)
        .finish();

    let schema_data = web::Data::new(schema);
    let database_backend = app_state.db_pool.backend_name();
    let auth_backend = app_state.auth_mode.as_str();
    let app_state_data = web::Data::new(app_state);

    let allowed_origins = allowed_origins_from_env();
    println!("Scryr running at http://{host}:{port}");
    println!("GraphQL API at http://{host}:{port}/graphql");
    println!("Using {database_backend} and {auth_backend} auth");
    if database_backend == "sqlite"
        && let Some(sqlite_path) = persistence::sqlite_path_from_env()
    {
        let display_path = sqlite_path.canonicalize().unwrap_or(sqlite_path);
        println!("SQLite database at {}", display_path.display());
    }

    let server = HttpServer::new(move || {
        App::new()
            .wrap(cors_for_allowed_origins(allowed_origins.clone()))
            .app_data(schema_data.clone())
            .app_data(app_state_data.clone())
            .app_data(web::JsonConfig::default().limit(8 * 1024 * 1024))
            .service(
                web::resource("/health")
                    .guard(guard::Get())
                    .to(health_handler),
            )
            .service(
                web::resource("/ready")
                    .guard(guard::Get())
                    .to(readiness_handler),
            )
            .service(graphql_resource())
            .service(
                web::resource("/playground")
                    .guard(guard::Get())
                    .to(playground_handler),
            )
            .default_service(web::route().guard(guard::Get()).to(map_ui_handler))
    })
    .bind((host.as_str(), port))?
    .run();
    Ok(server)
}

/// Route browser visits, GraphQL operations, and WebSocket upgrades separately.
fn graphql_resource() -> actix_web::Resource {
    web::resource("/graphql")
        .route(web::post().to(graphql_handler))
        .route(
            web::get()
                .guard(guard::fn_guard(|context| {
                    context
                        .head()
                        .headers()
                        .get(actix_web::http::header::UPGRADE)
                        .and_then(|value| value.to_str().ok())
                        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
                }))
                .to(graphql_ws),
        )
        .route(web::get().to(playground_handler))
}

/// Parse explicitly configured CORS origins.
pub(crate) fn allowed_origins_from_env() -> Vec<String> {
    env::var("CORS_ALLOWED_ORIGINS").map_or_else(
        |_| Vec::new(),
        |value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(std::string::ToString::to_string)
                .collect()
        },
    )
}

/// Build the CORS middleware for GraphQL API routes.
fn cors_for_allowed_origins(allowed_origins: Vec<String>) -> Cors {
    Cors::default()
        .allowed_origin_fn(move |origin, _req| {
            let origin = origin.to_str().ok();

            origin.is_some_and(|origin| {
                origin.starts_with("http://localhost:")
                    || origin.starts_with("http://127.0.0.1:")
                    || origin == "https://scryr.app"
                    || origin == "https://scryr-map.vercel.app"
                    || (origin.starts_with("https://") && origin.ends_with(".vercel.app"))
                    || allowed_origins.iter().any(|allowed| allowed == origin)
            })
        })
        .allowed_methods(vec!["GET", "POST", "OPTIONS"])
        .allowed_headers(vec![
            actix_web::http::header::AUTHORIZATION,
            actix_web::http::header::ACCEPT,
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::HeaderName::from_static("x-scryr-clerk-org-id"),
        ])
        .supports_credentials()
        .max_age(3600)
}

#[cfg(test)]
mod tests {
    use super::graphql_resource;
    use crate::roots::{MutationRoot, QueryRoot};
    use crate::state::{AppState, AuthMode};
    use actix_web::{App, http::StatusCode, test, web};
    use async_graphql::{EmptySubscription, Schema};
    use crystal_core::persistence::DatabasePool;

    #[actix_web::test]
    async fn graphql_route_serves_browser_post_and_websocket_requests()
    -> Result<(), Box<dyn std::error::Error>> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;
        let state = AppState {
            sample: None,
            auth_mode: AuthMode::Local,
            clerk_authorizer: None,
            clerk_client: None,
            db_pool: DatabasePool::Sqlite(pool.clone()),
        };
        let schema = Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
            .data(state.clone())
            .data(state.db_pool.clone())
            .finish();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(schema))
                .app_data(web::Data::new(state))
                .service(graphql_resource()),
        )
        .await;

        let browser = test::TestRequest::get()
            .uri("/graphql")
            .insert_header(("accept", "text/html"))
            .to_request();
        let response = test::call_service(&app, browser).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .ok_or("missing content type")?,
            "text/html; charset=utf-8"
        );
        let body = test::read_body(response).await;
        let html = std::str::from_utf8(&body)?;
        assert!(html.contains("GraphQLPlayground.init"));
        assert!(html.contains("/graphql"));

        let post = test::TestRequest::post()
            .uri("/graphql")
            .set_json(serde_json::json!({"query": "{ __typename }"}))
            .to_request();
        let response = test::call_service(&app, post).await;
        assert_eq!(response.status(), StatusCode::OK);
        let json: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(
            json,
            serde_json::json!({"data": {"__typename": "QueryRoot"}})
        );

        let websocket = test::TestRequest::get()
            .uri("/graphql")
            .insert_header(("connection", "Upgrade"))
            .insert_header(("upgrade", "WebSocket"))
            .insert_header(("sec-websocket-version", "13"))
            .insert_header(("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ=="))
            .insert_header(("sec-websocket-protocol", "graphql-transport-ws"))
            .to_request();
        let response = test::call_service(&app, websocket).await;
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(
            response
                .headers()
                .get("sec-websocket-protocol")
                .ok_or("missing websocket protocol")?,
            "graphql-transport-ws"
        );
        drop(response);
        pool.close().await;
        Ok(())
    }
}
