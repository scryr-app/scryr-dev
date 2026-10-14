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

/// Export the canonical GraphQL schema for frontend generation and compatibility checks.
#[must_use]
pub fn graphql_schema_sdl() -> String {
    async_graphql::Schema::build(
        roots::QueryRoot,
        roots::MutationRoot::default(),
        async_graphql::EmptySubscription,
    )
    .finish()
    .sdl()
}

#[cfg(test)]
mod schema_snapshot_tests {

    #[test]
    fn operational_card_and_collection_api_is_removed() {
        let sdl = super::graphql_schema_sdl();
        for removed in [
            "githubActions:",
            "sourceCodeUrl:",
            "description:",
            "version:",
            "language:",
            "frameworks:",
            "deployment:",
            "ownerTeam:",
            "cicdTool:",
            "manifestQuery(",
            "diagramMetrics(",
            "reportHistory(",
            "actionHistory(",
            "recordReport(",
            "recordActionRun(",
        ] {
            assert!(
                !sdl.contains(removed),
                "removed GraphQL field remains: {removed}"
            );
        }
        for retained in [
            "blocks(",
            "scryrMaps:",
            "manifestDocument(",
            "saveManifestDocument(",
        ] {
            assert!(
                sdl.contains(retained),
                "architecture API missing: {retained}"
            );
        }
    }

    #[test]
    fn graphql_schema_matches_snapshot() {
        let sdl = super::graphql_schema_sdl();
        let expected = include_str!("graphql_schema.snapshot.graphql");

        if expected.trim().is_empty() {
            println!("{sdl}");
        }

        assert_eq!(sdl.trim(), expected.trim());
    }
}
