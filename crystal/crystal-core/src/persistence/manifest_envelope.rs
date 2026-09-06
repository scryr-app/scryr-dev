//! Stored generated-manifest envelope parsing.

use crate::generated_manifest_envelope::GeneratedManifestEnvelope;
use serde_json::Value;

/// Flatten multiple stored manifest arrays into one JSON array.
pub(super) fn flatten_manifest_artifact_rows(rows: &[&(String, String)]) -> Result<Value, String> {
    let mut combined = Vec::new();

    for (artifact_key, content) in rows {
        let value = parse_manifest_blocks_value(artifact_key, content)?;
        let array = value.as_array().ok_or_else(|| {
            format!("Generated manifest artifact {artifact_key} did not contain a JSON array")
        })?;
        combined.extend(array.iter().cloned());
    }

    Ok(Value::Array(combined))
}

/// Parse and validate a stored manifest envelope, returning its manifest array.
pub(super) fn parse_manifest_blocks_value(
    artifact_key: &str,
    content: &str,
) -> Result<Value, String> {
    let envelope = serde_json::from_str::<GeneratedManifestEnvelope>(content).map_err(|error| {
        format!("Failed to parse generated manifest artifact {artifact_key} as JSON: {error}")
    })?;

    if envelope.files.is_empty() {
        return Err(format!(
            "Generated manifest artifact {artifact_key} must include at least one Python source file"
        ));
    }
    let manifests = envelope.require_manifests()?;
    if manifests.is_empty() {
        return Err(format!(
            "Generated manifest artifact {artifact_key} must include at least one manifest"
        ));
    }

    for file in &envelope.files {
        if file.path.trim().is_empty() {
            return Err(format!(
                "Generated manifest artifact {artifact_key} contains a source file with an empty path"
            ));
        }
        if file.content.trim().is_empty() {
            return Err(format!(
                "Generated manifest artifact {artifact_key} contains an empty Python source file for {}",
                file.path
            ));
        }
    }

    for manifest in manifests {
        if manifest.line_number().is_none() {
            let name = manifest.name.as_deref().unwrap_or("<unknown>");
            return Err(format!(
                "Generated manifest artifact {artifact_key} is missing line_number for manifest {name}"
            ));
        }
    }

    Ok(Value::Array(envelope.into_manifest_values()))
}

#[cfg(test)]
mod tests {
    use super::{flatten_manifest_artifact_rows, parse_manifest_blocks_value};
    use serde_json::json;

    #[test]
    fn flatten_manifest_artifact_rows_combines_arrays() -> Result<(), String> {
        let first = json!({
            "files": [{"path": "index.scry", "content": "api = Manifest(name='API')"}],
            "manifests": [{"name": "API", "line_number": 1}]
        })
        .to_string();
        let second = json!({
            "files": [{"path": "worker.scry", "content": "worker = Manifest(name='Worker')"}],
            "manifests": [{"name": "Worker", "line_number": 1}]
        })
        .to_string();
        let rows = [("api".to_string(), first), ("worker".to_string(), second)];
        let refs = rows.iter().collect::<Vec<_>>();

        let combined = flatten_manifest_artifact_rows(&refs)?;

        assert_eq!(combined.as_array().map_or(0, Vec::len), 2);
        Ok(())
    }

    #[test]
    fn parse_manifest_blocks_value_supports_manifest_envelope() -> Result<(), String> {
        let value = parse_manifest_blocks_value(
            "mern",
            r#"{
                "files": [{"path": "index.scry", "content": "api = Manifest(name='API')"}],
                "manifests": [{"name": "API", "line_number": 1}]
            }"#,
        )?;

        assert!(value.is_array());
        assert_eq!(value.as_array().map_or(0, Vec::len), 1);
        Ok(())
    }

    #[test]
    fn parse_manifest_blocks_value_rejects_legacy_array_payloads() {
        let error = parse_manifest_blocks_value("mern", r#"[{"name":"React"}]"#).err();

        assert!(
            error
                .as_deref()
                .is_some_and(|message| message.contains("Failed to parse generated manifest"))
        );
    }

    #[test]
    fn generated_manifest_value_artifacts_require_source_files_and_line_numbers() {
        let invalid_artifacts = [
            (
                "missing_files",
                r#"{"files":[],"manifests":[{"name":"API","line_number":1}]}"#,
                "must include at least one Python source file",
            ),
            (
                "missing_manifests",
                r#"{"files":[{"path":"index.scry","content":"api = Manifest()"}],"manifests":[]}"#,
                "must include at least one manifest",
            ),
            (
                "empty_file",
                r#"{"files":[{"path":"index.scry","content":""}],"manifests":[{"name":"API","line_number":1}]}"#,
                "contains an empty Python source file",
            ),
            (
                "missing_line",
                r#"{"files":[{"path":"index.scry","content":"api = Manifest()"}],"manifests":[{"name":"API"}]}"#,
                "missing line_number",
            ),
        ];

        for (artifact_key, content, expected_message) in invalid_artifacts {
            let validation = parse_manifest_blocks_value(artifact_key, content);
            assert!(
                validation
                    .err()
                    .is_some_and(|message| message.contains(expected_message)),
                "expected validation to reject {artifact_key} with {expected_message}"
            );
        }
    }
}
