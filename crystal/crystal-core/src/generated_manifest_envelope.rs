//! Shared generated manifest envelope models.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A single manifest source file stored alongside generated manifests.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestSourceFile {
    /// Relative path within the captured manifest source root.
    pub path: String,
    /// Source file contents.
    pub content: String,
}

/// Persisted value-artifact envelope for generated manifests and their metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneratedManifestEnvelope {
    /// Source files required to reconstruct the generated manifests.
    #[serde(default)]
    pub files: Vec<ManifestSourceFile>,
    /// Top-level manifest records emitted by the adapter.
    pub manifests: Option<Vec<GeneratedManifestRecord>>,
    /// Forge records preserved in scoped map artifacts.
    #[serde(default)]
    pub forges: Vec<Value>,
    /// Diagram records that define map-specific artifacts.
    pub diagrams: Option<Vec<GeneratedDiagramRecord>>,
    /// Future envelope fields preserved at the serialization edge.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl GeneratedManifestEnvelope {
    /// Build an envelope from raw adapter values.
    ///
    /// # Errors
    ///
    /// Returns an error if a manifest or diagram value does not have the
    /// expected object shape.
    pub fn from_adapter_values(
        files: Vec<ManifestSourceFile>,
        manifests: Vec<Value>,
        forges: Vec<Value>,
        diagrams: Vec<Value>,
    ) -> Result<Self, String> {
        Ok(Self {
            files,
            manifests: Some(
                manifests
                    .into_iter()
                    .map(GeneratedManifestRecord::from_value)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            forges,
            diagrams: Some(
                diagrams
                    .into_iter()
                    .map(GeneratedDiagramRecord::from_value)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            extra: Map::new(),
        })
    }

    /// Return manifest records or a validation error.
    ///
    /// # Errors
    ///
    /// Returns an error if the envelope has no manifest array.
    pub fn require_manifests(&self) -> Result<&[GeneratedManifestRecord], String> {
        self.manifests
            .as_deref()
            .ok_or_else(|| "Generated manifest JSON did not include a manifest array".to_string())
    }

    /// Consume the envelope and return its manifest records as raw JSON values.
    #[must_use]
    pub fn into_manifest_values(self) -> Vec<Value> {
        self.manifests
            .unwrap_or_default()
            .into_iter()
            .map(GeneratedManifestRecord::into_value)
            .collect()
    }
}

/// Top-level Manifest record emitted by Python.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneratedManifestRecord {
    /// Manifest display name used by Diagram references.
    pub name: Option<String>,
    /// Remaining manifest payload fields.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl GeneratedManifestRecord {
    /// Convert a raw JSON manifest object into a typed record.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a JSON object.
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value)
            .map_err(|error| format!("Generated Manifest record was malformed: {error}"))
    }

    /// Return the source line number attached to this manifest record.
    #[must_use]
    pub fn line_number(&self) -> Option<i64> {
        self.extra.get("line_number").and_then(Value::as_i64)
    }

    /// Convert this record back into a JSON value.
    #[must_use]
    pub fn into_value(self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// Top-level Diagram record emitted by Python.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeneratedDiagramRecord {
    /// Diagram display name.
    pub name: Option<String>,
    /// Python variable name used as stable map id.
    pub variable_name: Option<String>,
    /// Manifest references contained in this Diagram.
    pub manifests: Option<Vec<DiagramManifestReference>>,
    /// Remaining diagram payload fields.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl GeneratedDiagramRecord {
    /// Convert a raw JSON diagram object into a typed record.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a JSON object.
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value)
            .map_err(|error| format!("Generated Diagram record was malformed: {error}"))
    }
}

/// Manifest reference embedded in a Diagram record.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiagramManifestReference {
    /// Referenced Manifest display name.
    pub name: Option<String>,
    /// Remaining reference payload fields.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
