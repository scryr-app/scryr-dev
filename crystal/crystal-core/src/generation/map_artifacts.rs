//! Generated map artifact splitting for upload.

use crate::generated_manifest_envelope::{
    GeneratedDiagramRecord, GeneratedManifestEnvelope, GeneratedManifestRecord,
};
use std::collections::HashSet;

/// One generated value artifact to upload for a Scryr map.
pub struct GeneratedMapArtifact {
    /// Generated manifest value artifact content.
    pub manifest_json: String,
    /// Persisted artifact key for the generated value artifact.
    pub artifact_key: String,
    /// Map metadata derived from the declared Diagram.
    pub map_metadata: GeneratedMapMetadata,
}

/// Upload identity and display metadata derived from the generated Diagram.
pub struct GeneratedMapMetadata {
    /// Stable identifier for the uploaded map.
    pub scry_identifier: String,
    /// Human-readable map name.
    pub name: String,
}

/// Derive upload artifacts from every declared Diagram in a generated envelope.
pub(crate) fn map_artifacts_from_manifest_json(
    manifest_json: &str,
    fallback_identifier: &str,
) -> Result<Vec<GeneratedMapArtifact>, String> {
    let envelope = serde_json::from_str::<GeneratedManifestEnvelope>(manifest_json)
        .map_err(|error| format!("Generated manifest JSON was malformed: {error}"))?;
    let Some(diagrams) = envelope.diagrams.as_deref() else {
        return Ok(vec![fallback_map_artifact(
            manifest_json,
            fallback_identifier,
        )]);
    };

    if diagrams.is_empty() {
        return Ok(vec![fallback_map_artifact(
            manifest_json,
            fallback_identifier,
        )]);
    }

    let mut seen_identifiers = HashSet::new();
    let mut artifacts = Vec::new();
    for diagram in diagrams {
        let map_metadata = map_metadata_from_diagram(diagram)?;
        if !seen_identifiers.insert(map_metadata.scry_identifier.clone()) {
            return Err(format!(
                "Generated manifest JSON defined duplicate Diagram variable_name `{}`",
                map_metadata.scry_identifier
            ));
        }

        let manifest_json =
            scoped_manifest_json_for_diagram(&envelope, diagram, &map_metadata.name)?;
        let artifact_key = artifact_key_for_diagram(fallback_identifier, &map_metadata);
        artifacts.push(GeneratedMapArtifact {
            manifest_json,
            artifact_key,
            map_metadata,
        });
    }

    Ok(artifacts)
}

/// Build the legacy single-map artifact for envelopes without Diagram records.
fn fallback_map_artifact(manifest_json: &str, fallback_identifier: &str) -> GeneratedMapArtifact {
    GeneratedMapArtifact {
        manifest_json: manifest_json.to_string(),
        artifact_key: fallback_identifier.to_string(),
        map_metadata: GeneratedMapMetadata {
            scry_identifier: fallback_identifier.to_string(),
            name: fallback_identifier.to_string(),
        },
    }
}

/// Return upload metadata from one serialized Diagram wrapper.
fn map_metadata_from_diagram(
    diagram: &GeneratedDiagramRecord,
) -> Result<GeneratedMapMetadata, String> {
    let scry_identifier = diagram
        .variable_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Generated Diagram did not include variable_name".to_string())?;
    let name = diagram
        .name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Generated Diagram {scry_identifier} did not include a name"))?;

    Ok(GeneratedMapMetadata {
        scry_identifier: scry_identifier.to_string(),
        name: name.to_string(),
    })
}

/// Build a generated envelope containing only one Diagram's manifest blocks.
fn scoped_manifest_json_for_diagram(
    envelope: &GeneratedManifestEnvelope,
    diagram: &GeneratedDiagramRecord,
    diagram_name: &str,
) -> Result<String, String> {
    let mut scoped = envelope.clone();
    scoped.diagrams = Some(vec![diagram.clone()]);
    scoped.manifests = Some(scoped_manifests_for_diagram(
        envelope,
        diagram,
        diagram_name,
    )?);

    serde_json::to_string_pretty(&scoped)
        .map_err(|error| format!("Failed to serialize scoped diagram artifact: {error}"))
}

/// Return top-level manifest records referenced by one Diagram.
fn scoped_manifests_for_diagram(
    envelope: &GeneratedManifestEnvelope,
    diagram: &GeneratedDiagramRecord,
    diagram_name: &str,
) -> Result<Vec<GeneratedManifestRecord>, String> {
    let all_manifests = envelope
        .manifests
        .as_deref()
        .ok_or_else(|| "Generated manifest JSON did not include a manifest array".to_string())?;
    let diagram_manifests = diagram
        .manifests
        .as_deref()
        .ok_or_else(|| format!("Generated Diagram {diagram_name} did not include manifests"))?;
    if diagram_manifests.is_empty() {
        return Err(format!(
            "Generated Diagram {diagram_name} did not include any manifests"
        ));
    }

    let mut seen_names = HashSet::new();
    let mut scoped_manifests = Vec::new();
    for diagram_manifest in diagram_manifests {
        let name = diagram_manifest.name.as_deref().ok_or_else(|| {
            format!("Generated Diagram {diagram_name} referenced a manifest without a name")
        })?;
        if !seen_names.insert(name.to_string()) {
            continue;
        }

        let Some(top_level_manifest) = all_manifests
            .iter()
            .find(|manifest| manifest.name.as_deref() == Some(name))
        else {
            return Err(format!(
                "Generated Diagram {diagram_name} references manifest {name}, but that manifest was not emitted as a public top-level Manifest"
            ));
        };
        scoped_manifests.push(top_level_manifest.clone());
    }

    Ok(scoped_manifests)
}

/// Return the persisted artifact key for a diagram.
fn artifact_key_for_diagram(fallback_identifier: &str, metadata: &GeneratedMapMetadata) -> String {
    let primary_diagram_identifier = format!("{fallback_identifier}_diagram");
    if metadata.scry_identifier == primary_diagram_identifier {
        fallback_identifier.to_string()
    } else {
        metadata.scry_identifier.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::map_artifacts_from_manifest_json;
    use serde_json::{Value, json};

    #[test]
    fn multi_diagram_manifest_json_splits_into_scoped_map_artifacts() -> Result<(), String> {
        let manifest_json = json!({
            "files": [{"path": "index.scry", "content": "web = Manifest(name='Web')"}],
            "manifests": [
                {"name": "Web", "line_number": 1},
                {"name": "API", "line_number": 2},
                {"name": "Worker", "line_number": 3}
            ],
            "forges": [],
            "diagrams": [
                {
                    "name": "Main",
                    "variable_name": "main_diagram",
                    "manifests": [{"name": "Web"}, {"name": "API"}]
                },
                {
                    "name": "Jobs",
                    "variable_name": "jobs_diagram",
                    "manifests": [{"name": "Worker"}]
                }
            ]
        })
        .to_string();

        let artifacts = map_artifacts_from_manifest_json(&manifest_json, "main")?;

        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].artifact_key, "main");
        assert_eq!(artifacts[1].artifact_key, "jobs_diagram");

        let main_content = serde_json::from_str::<Value>(&artifacts[0].manifest_json)
            .map_err(|error| format!("main artifact should be JSON: {error}"))?;
        let main_manifests = main_content
            .get("manifests")
            .and_then(Value::as_array)
            .ok_or_else(|| "main artifact should include manifests".to_string())?;
        assert_eq!(main_manifests.len(), 2);
        assert_eq!(main_manifests[0]["name"], "Web");
        assert_eq!(main_manifests[1]["name"], "API");

        let jobs_content = serde_json::from_str::<Value>(&artifacts[1].manifest_json)
            .map_err(|error| format!("jobs artifact should be JSON: {error}"))?;
        let jobs_manifests = jobs_content
            .get("manifests")
            .and_then(Value::as_array)
            .ok_or_else(|| "jobs artifact should include manifests".to_string())?;
        assert_eq!(jobs_manifests.len(), 1);
        assert_eq!(jobs_manifests[0]["name"], "Worker");

        Ok(())
    }
}
