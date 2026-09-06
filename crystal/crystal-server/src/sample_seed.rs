//! Startup seeding for bundled local sample maps.

use crate::auth::local_dev_manifest_request_context;
use crate::static_assets;
use crystal_core::manifest::UpsertGeneratedManifestInput;
use crystal_core::persistence::persist_generated_manifest;
use crystal_core::persistence::{self, ArtifactKind, DatabasePool};
use serde_json::Value;

/// Seed bundled sample maps into local storage when they are missing.
///
/// # Errors
///
/// Returns an error if a bundled sample is malformed or cannot be persisted.
pub(crate) async fn seed_missing_local_samples(pool: &DatabasePool) -> Result<(), String> {
    let request_context = local_dev_manifest_request_context();

    for sample in static_assets::sample_manifests() {
        match persistence::read_generated_manifest_json(
            pool,
            &request_context.clerk_org_id,
            Some(sample.name),
        )
        .await
        .map_err(String::from)
        {
            Ok(_) => continue,
            Err(error) if error.contains("Generated manifest artifact not found") => {}
            Err(error) if error.contains("Generated manifest artifacts not found") => {}
            Err(error) if should_repair_bundled_sample(&error) => {}
            Err(error) => {
                return Err(format!(
                    "Failed to check bundled sample {} before seeding: {error}",
                    sample.name
                ));
            }
        }

        let metadata = sample_metadata(sample.name, sample.json)?;
        persist_generated_manifest(
            pool,
            &UpsertGeneratedManifestInput {
                artifact_kind: ArtifactKind::Value,
                artifact_key: sample.name.to_string(),
                folder_path: Some(sample.name.to_string()),
                file_name: Some("index.scry".to_string()),
                scry_identifier: Some(sample.name.to_string()),
                name: Some(metadata.name),
                git_commit_sha: None,
                content: sample.json.to_string(),
            },
            &request_context,
        )
        .await
        .map_err(|error| format!("Failed to seed bundled sample {}: {error}", sample.name))?;
    }

    Ok(())
}

/// Return whether an existing bundled sample row should be replaced.
fn should_repair_bundled_sample(error: &str) -> bool {
    error.contains("Failed to parse generated manifest artifact")
        || error.contains("must include at least one Python source file")
        || error.contains("must include at least one manifest")
        || error.contains("missing line_number")
}

/// Display metadata extracted from a generated sample envelope.
struct SampleMetadata {
    /// Human-readable map name.
    name: String,
}

/// Extract map metadata from a generated sample envelope.
fn sample_metadata(sample_name: &str, json: &str) -> Result<SampleMetadata, String> {
    let envelope = serde_json::from_str::<Value>(json)
        .map_err(|error| format!("Bundled sample {sample_name} JSON is malformed: {error}"))?;
    let first_diagram = envelope
        .get("diagrams")
        .and_then(Value::as_array)
        .and_then(|diagrams| diagrams.first());
    let name = first_diagram
        .and_then(|diagram| diagram.get("name"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(sample_name);

    Ok(SampleMetadata {
        name: name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{sample_metadata, should_repair_bundled_sample};

    #[test]
    fn sample_metadata_prefers_first_diagram() -> Result<(), String> {
        let metadata = sample_metadata(
            "mern",
            r#"{"diagrams":[{"variable_name":"system_diagram","name":"System"}]}"#,
        )?;

        assert_eq!(metadata.name, "System");
        Ok(())
    }

    #[test]
    fn sample_metadata_falls_back_to_sample_name() -> Result<(), String> {
        let metadata = sample_metadata("mern", r#"{"manifests":[]}"#)?;

        assert_eq!(metadata.name, "mern");
        Ok(())
    }

    #[test]
    fn bundled_sample_repair_detects_malformed_persisted_rows() {
        assert!(should_repair_bundled_sample(
            "Failed to parse generated manifest artifact mern as JSON"
        ));
        assert!(should_repair_bundled_sample(
            "Generated manifest artifact mern contains manifest React missing line_number"
        ));
        assert!(!should_repair_bundled_sample(
            "Failed to load generated manifest artifact for sample mern: database locked"
        ));
    }
}
