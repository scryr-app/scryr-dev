//! GraphQL health status type.

use async_graphql::SimpleObject;
use serde::Serialize;

/// Health check result exposed over GraphQL and HTTP.
#[derive(SimpleObject, Serialize, Clone)]
pub struct HealthStatus {
    /// Overall service status.
    pub status: String,
    /// Whether the database query succeeded.
    pub database_ok: bool,
    /// Details about the database health check result.
    pub database_message: String,
}
