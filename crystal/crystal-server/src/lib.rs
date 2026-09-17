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
mod samples;
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
        if std::env::var_os("SCRYR_UPDATE_GRAPHQL_SCHEMA").is_some() {
            std::fs::write(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/graphql_schema.snapshot.graphql"
                ),
                &sdl,
            )
            .unwrap_or_else(|error| {
                eprintln!("Cannot write schema snapshot: {error}");
                std::process::exit(1)
            });
            return;
        }
        let expected = include_str!("graphql_schema.snapshot.graphql");

        if expected.trim().is_empty() {
            println!("{sdl}");
        }

        assert_eq!(sdl.trim(), expected.trim());
    }
}
