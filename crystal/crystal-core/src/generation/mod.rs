//! Pure manifest transformations and artifact rendering.

mod map_artifacts;
mod renderers;

pub use map_artifacts::{GeneratedMapArtifact, GeneratedMapMetadata};

/// Derive upload artifacts from every declared Diagram in a generated envelope.///
/// # Errors
///
/// Returns a validation error if the manifest or selected Forge is invalid.
pub fn map_artifacts_from_manifest_json(
    manifest_json: &str,
    fallback_identifier: &str,
) -> Result<Vec<GeneratedMapArtifact>, crate::Error> {
    map_artifacts::map_artifacts_from_manifest_json(manifest_json, fallback_identifier)
        .map_err(crate::Error::Validation)
}

/// Render Docker Compose YAML from known service tools in the selected Forge.///
/// # Errors
///
/// Returns a validation error if the manifest or selected Forge is invalid.
pub fn render_compose_yaml(
    manifest_json: &str,
    forge_selector: Option<&str>,
) -> Result<String, crate::Error> {
    renderers::compose::render_compose_yaml(manifest_json, forge_selector)
        .map_err(crate::Error::Validation)
}

/// Render a devcontainer.json from the selected Forge.///
/// # Errors
///
/// Returns a validation error if the manifest or selected Forge is invalid.
pub fn render_devcontainer_json(
    manifest_json: &str,
    forge_selector: Option<&str>,
) -> Result<String, crate::Error> {
    renderers::devcontainer::render_devcontainer_json(manifest_json, forge_selector)
        .map_err(crate::Error::Validation)
}

/// Render a selected forge from the generated manifest envelope as mise.toml.///
/// # Errors
///
/// Returns a validation error if the manifest or selected Forge is invalid.
pub fn render_mise_toml(
    manifest_json: &str,
    forge_selector: Option<&str>,
) -> Result<String, crate::Error> {
    renderers::mise::render_mise_toml(manifest_json, forge_selector)
        .map_err(crate::Error::Validation)
}
