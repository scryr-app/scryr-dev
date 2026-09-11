//! Scryr GraphQL server.
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::useless_let_if_seq)]

mod auth;
/// Source editing and explicit local workspace registration.
pub mod editor;
mod generated_manifest_graphql;
mod health;
mod http_handlers;
mod roots;
mod runtime_metrics;
/// GraphQL HTTP server bootstrap.
pub mod server;
/// Shared server state and configuration.
pub mod state;
mod static_assets;

pub use state::ServerArgs;

#[cfg(test)]
mod schema_snapshot_tests {
    use super::roots::{MutationRoot, QueryRoot};
    use async_graphql::{EmptySubscription, Schema};

    #[test]
    fn graphql_schema_matches_snapshot() {
        let schema = Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription).finish();
        let sdl = schema.sdl();
        let expected = include_str!("graphql_schema.snapshot.graphql");

        if expected.trim().is_empty() {
            println!("{sdl}");
        }

        assert_eq!(sdl.trim(), expected.trim());
    }
}
