//! Sprite execution environment configuration.

use std::path::PathBuf;

/// sprites.dev execution target for untrusted manifest code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpriteExecutionEnvironment {
    /// Sprite CLI executable.
    pub(super) sprite_executable: PathBuf,
    /// Sprite name selected for remote execution.
    pub(super) sprite_name: String,
    /// Optional Sprites organization name.
    pub(super) organization: Option<String>,
}

impl SpriteExecutionEnvironment {
    /// Build a sprites.dev execution target.
    pub(crate) const fn new(
        sprite_executable: PathBuf,
        sprite_name: String,
        organization: Option<String>,
    ) -> Self {
        Self {
            sprite_executable,
            sprite_name,
            organization,
        }
    }
}
