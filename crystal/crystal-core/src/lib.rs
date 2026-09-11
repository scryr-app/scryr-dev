//! GraphQL-backed manifest models, pure artifact generation, and persistence for Scryr.
#![allow(clippy::missing_docs_in_private_items)]

/// GitHub Actions run and status history models.
pub mod action_history;
/// Typed operational report observations.
pub mod reports;

mod error;
pub use error::Error;

/// Shared generated manifest envelope models.
pub mod generated_manifest_envelope;
/// Pure manifest transformations and artifact renderers.
pub mod generation;
/// Canonical GraphQL object, input, and enum types.
pub mod graphql_types;
/// Shared GraphQL upload input and authenticated principal models.
pub mod manifest;
/// Storage connections, reads, and writes.
pub mod persistence;

/// Shared TOML integration credential loading and validation.
pub mod integration_secrets;
