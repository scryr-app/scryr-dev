//! GraphQL server command.

use crystal_server::ServerArgs;

/// Run the embedded Scryr GraphQL server.
pub(super) async fn run(args: &ServerArgs) -> Result<(), String> {
    crystal_server::server::run(args.clone())
        .await
        .map_err(|error| format!("GraphQL server failed: {error}"))
}
