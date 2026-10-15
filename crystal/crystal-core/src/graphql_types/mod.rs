//! GraphQL object and enum types shared by the server and CLI.

mod block;
mod health;
mod json_access;
mod map;

pub use block::{Block, Link};
pub use health::HealthStatus;
pub use map::ScryrMap;

// Canonical upload types also live in the shared manifest model module.
pub use crate::manifest::{
    ArtifactKind, UpsertGeneratedManifestInput, UpsertGeneratedManifestPayload,
};
