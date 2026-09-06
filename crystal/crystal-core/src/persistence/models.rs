//! Public and row models for generated manifest persistence.

use sqlx::SqlitePool;
use std::sync::Arc;

pub use crate::graphql_types::ScryrMap as GeneratedManifestMap;
pub use crate::manifest::ArtifactKind;

/// Database row shape used when listing generated maps.
pub(super) type GeneratedManifestMapRow = (
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    Option<String>,
    String,
);

/// SQL database pool used for generated manifest persistence.
#[derive(Clone)]
pub enum DatabasePool {
    /// Hosted Turso/libSQL storage.
    Turso(Arc<libsql::Database>),
    /// Zero-setup local `SQLite` storage.
    Sqlite(SqlitePool),
}

impl DatabasePool {
    /// Return a short human-readable backend name.
    #[must_use]
    pub const fn backend_name(&self) -> &'static str {
        match self {
            Self::Turso(_) => "turso",
            Self::Sqlite(_) => "sqlite",
        }
    }
}
