//! CLI for generating manifest artifacts.

/// Application orchestration for the CLI workflow.
mod app;
/// Command-line argument parsing for `scryr`.
mod args;
/// Interactive Clerk OAuth helpers.
mod auth;
/// Top-level command dispatch and command-specific workflows.
mod commands;
/// GraphQL HTTP client for generated manifest persistence.
mod graphql_client;
/// Manifest path resolution and artifact key derivation helpers.
mod manifest_paths;
/// Embedded Python execution helpers for manifest generation.
mod manifest_python;
/// Manifest source discovery and source metadata helpers.
mod manifest_source;
/// sprites.dev execution helpers for untrusted manifest generation.
mod manifest_sprite;
/// Scryr state directory resolution helpers.
mod scryr_dir;
/// uv executable installation and resolution helpers.
mod uv;

use crate::args::Args;
use clap::Parser;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    app::run(&Args::parse())
        .await
        .map_err(std::io::Error::other)
}
