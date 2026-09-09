//! Compile-time embedded map UI assets.

/// A single embedded frontend asset.
pub(crate) struct EmbeddedAsset {
    /// Request path relative to the Vite dist root.
    pub(crate) path: &'static str,
    /// HTTP content type for the asset.
    pub(crate) content_type: &'static str,
    /// Static asset bytes embedded into the server binary.
    pub(crate) bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/embedded_map_assets.rs"));

/// Look up a frontend asset by path.
pub(crate) fn get(path: &str) -> Option<&'static EmbeddedAsset> {
    ASSETS.iter().find(|asset| asset.path == path)
}

/// Return whether a frontend build was embedded at compile time.
pub(crate) fn has_map_ui() -> bool {
    get("index.html").is_some()
}
