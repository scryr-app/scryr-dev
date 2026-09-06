//! Health checks for HTTP and GraphQL.

use crate::state::AppState;
use actix_web::{HttpResponse, web};
use crystal_core::graphql_types::HealthStatus;
use crystal_core::persistence::DatabasePool;
use sqlx::Row;

/// Liveness endpoint — returns 200 OK when the process is serving HTTP.
pub(crate) async fn health_handler(app_state: web::Data<AppState>) -> HttpResponse {
    let health = run_healthcheck(app_state.get_ref()).await;
    HttpResponse::Ok().json(HealthStatus {
        status: "ok".to_string(),
        database_ok: health.database_ok,
        database_message: health.database_message,
    })
}

/// Readiness endpoint — returns 200 OK only when dependencies are healthy.
pub(crate) async fn readiness_handler(app_state: web::Data<AppState>) -> HttpResponse {
    let health = run_healthcheck(app_state.get_ref()).await;
    let status_code = if health.database_ok {
        actix_web::http::StatusCode::OK
    } else {
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    };

    HttpResponse::build(status_code).json(health)
}

/// Runs the shared application health check, including a simple database query.
pub(crate) async fn run_healthcheck(state: &AppState) -> HealthStatus {
    let backend_name = state.db_pool.backend_name();
    let result = match &state.db_pool {
        DatabasePool::Turso(database) => run_turso_healthcheck(database).await,
        DatabasePool::Sqlite(pool) => sqlx::query("SELECT 1 AS healthcheck")
            .fetch_one(pool)
            .await
            .map(|row| row.try_get::<i32, _>("healthcheck"))
            .map(|result| result.map_err(|error| error.to_string()))
            .map_err(|error| error.to_string()),
    };

    match result {
        Ok(Ok(1)) => HealthStatus {
            status: "ok".to_string(),
            database_ok: true,
            database_message: format!("{backend_name} database query succeeded"),
        },
        Ok(Ok(_)) => HealthStatus {
            status: "error".to_string(),
            database_ok: false,
            database_message: "database healthcheck returned an unexpected value".to_string(),
        },
        Ok(Err(error)) => HealthStatus {
            status: "error".to_string(),
            database_ok: false,
            database_message: format!("database row parse failed: {error}"),
        },
        Err(error) => HealthStatus {
            status: "error".to_string(),
            database_ok: false,
            database_message: format!("database query failed: {error}"),
        },
    }
}

/// Run a simple Turso/libSQL healthcheck query.
async fn run_turso_healthcheck(database: &libsql::Database) -> Result<Result<i32, String>, String> {
    let connection = database
        .connect()
        .map_err(|error| format!("failed to open Turso healthcheck connection: {error}"))?;
    let mut rows = connection
        .query("SELECT 1 AS healthcheck", ())
        .await
        .map_err(|error| format!("failed to run Turso healthcheck query: {error}"))?;
    let row = rows
        .next()
        .await
        .map_err(|error| format!("failed to read Turso healthcheck row: {error}"))?
        .ok_or_else(|| "Turso healthcheck returned no rows".to_string())?;

    Ok(row
        .get::<i32>(0)
        .map_err(|error| format!("failed to parse Turso healthcheck value: {error}")))
}
