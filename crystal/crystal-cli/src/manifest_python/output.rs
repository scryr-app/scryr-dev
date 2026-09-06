//! Formatting Python adapter output into CLI artifacts.

use crate::manifest_source::{collect_manifest_source_files, manifest_line_numbers};
use crystal_core::generated_manifest_envelope::GeneratedManifestEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Output mode for the Python adapter entrypoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManifestPythonMode {
    /// Emit serialized manifest blocks from a module.
    Json,
    /// Emit field type metadata from a module.
    Types,
    /// Emit the manifest model JSON schema.
    Schema,
}

impl ManifestPythonMode {
    /// Command-line flag consumed by the Python adapter entrypoint.
    pub(crate) const fn as_flag(self) -> &'static str {
        match self {
            Self::Json => "--json",
            Self::Types => "--types",
            Self::Schema => "--schema",
        }
    }
}

/// One manifest item returned by the Python execution adapter.
#[derive(Debug, Deserialize)]
pub(crate) struct PythonScryrValue {
    /// Scryr construct kind returned by the Python adapter.
    kind: Option<String>,
    /// Python variable name that held the Scryr top-level instance.
    variable_name: String,
    /// Serialized Pydantic manifest object.
    manifest: Option<Value>,
    /// Serialized Pydantic forge object.
    forge: Option<Value>,
    /// Serialized Pydantic diagram object.
    diagram: Option<Value>,
}

/// Parsed Python adapter output for one requested mode.
pub(crate) enum PythonAdapterOutput {
    /// Serialized top-level Scryr values.
    Json(Vec<PythonScryrValue>),
    /// Manifest field type metadata.
    Types(Vec<Value>),
    /// Manifest model JSON schema.
    Schema(Value),
}

impl PythonAdapterOutput {
    /// Parse adapter stdout according to the requested mode.
    pub(crate) fn parse(
        manifest_file: &Path,
        mode: ManifestPythonMode,
        stdout: &str,
    ) -> Result<Self, String> {
        match mode {
            ManifestPythonMode::Json => {
                serde_json::from_str::<Vec<PythonScryrValue>>(stdout)
                    .map(Self::Json)
                    .map_err(|error| {
                        format!(
                            "Python manifest adapter returned malformed Scryr data for {}: {error}",
                            manifest_file.display()
                        )
                    })
            }
            ManifestPythonMode::Types => serde_json::from_str::<Vec<Value>>(stdout)
                .map(Self::Types)
                .map_err(|error| {
                    format!(
                        "Python manifest adapter returned malformed type metadata for {}: {error}",
                        manifest_file.display()
                    )
                }),
            ManifestPythonMode::Schema => serde_json::from_str::<Value>(stdout)
                .map(Self::Schema)
                .map_err(|error| {
                    format!(
                        "Python manifest adapter returned malformed manifest schema for {}: {error}",
                        manifest_file.display()
                    )
                }),
        }
    }
}

/// Format one JSON payload returned by a manifest execution adapter.
pub(crate) fn format_manifest_adapter_output(
    manifest_file: &Path,
    mode: ManifestPythonMode,
    python_output: PythonAdapterOutput,
) -> Result<String, String> {
    match (mode, python_output) {
        (ManifestPythonMode::Json, PythonAdapterOutput::Json(values)) => {
            build_manifest_json_artifact(manifest_file, values)
        }
        (ManifestPythonMode::Types, PythonAdapterOutput::Types(types)) => {
            format_manifest_types(manifest_file, &types)
        }
        (ManifestPythonMode::Schema, PythonAdapterOutput::Schema(schema)) => {
            format_manifest_schema(&schema)
        }
        _ => Err("Python manifest adapter output did not match the requested mode".to_string()),
    }
}
/// Build the persisted manifest JSON envelope from Python values and Rust metadata.
fn build_manifest_json_artifact(
    manifest_file: &Path,
    values: Vec<PythonScryrValue>,
) -> Result<String, String> {
    if values.is_empty() {
        return Err(format!(
            "Manifest file {} did not define any public Scryr top-level instances",
            manifest_file.display()
        ));
    }

    let line_numbers = manifest_line_numbers(manifest_file)?;
    let mut manifests = Vec::new();
    let mut forges = Vec::new();
    let mut diagrams = Vec::new();

    for value in values {
        match value.kind.as_deref() {
            Some("forge") => {
                let forge = attach_scryr_metadata(
                    value.forge.ok_or_else(|| {
                        format!(
                            "Forge variable {} in {} did not include a forge payload",
                            value.variable_name,
                            manifest_file.display()
                        )
                    })?,
                    value.variable_name,
                    &line_numbers,
                    manifest_file,
                    "Forge",
                )?;
                forges.push(forge);
            }
            Some("diagram") => {
                let diagram = attach_scryr_metadata(
                    value.diagram.ok_or_else(|| {
                        format!(
                            "Diagram variable {} in {} did not include a diagram payload",
                            value.variable_name,
                            manifest_file.display()
                        )
                    })?,
                    value.variable_name,
                    &line_numbers,
                    manifest_file,
                    "Diagram",
                )?;
                diagrams.push(diagram);
            }
            _ => {
                let manifest = attach_scryr_metadata(
                    value.manifest.ok_or_else(|| {
                        format!(
                            "Manifest variable {} in {} did not include a manifest payload",
                            value.variable_name,
                            manifest_file.display()
                        )
                    })?,
                    value.variable_name,
                    &line_numbers,
                    manifest_file,
                    "Manifest",
                )?;
                manifests.push(manifest);
            }
        }
    }

    let envelope = GeneratedManifestEnvelope::from_adapter_values(
        collect_manifest_source_files(manifest_file)?,
        manifests,
        forges,
        diagrams,
    )?;
    format_json(&envelope)
}

/// Validate and format manifest type metadata from the Python adapter.
fn format_manifest_types(manifest_file: &Path, types: &[Value]) -> Result<String, String> {
    if types.is_empty() {
        return Err(format!(
            "Manifest file {} did not define any public Manifest instances",
            manifest_file.display()
        ));
    }

    format_json(&types)
}

/// Validate and format the manifest model schema from the Python adapter.
fn format_manifest_schema(schema: &Value) -> Result<String, String> {
    if !schema.is_object() {
        return Err("Python manifest adapter returned malformed manifest schema".to_string());
    }

    format_json(&schema)
}

/// Attach Rust-computed source metadata to a serialized top-level Scryr object.
fn attach_scryr_metadata(
    value: Value,
    variable_name: String,
    line_numbers: &HashMap<String, usize>,
    manifest_file: &Path,
    construct_name: &str,
) -> Result<Value, String> {
    let Value::Object(mut manifest) = value else {
        return Err(format!(
            "{construct_name} variable {} in {} did not serialize to a JSON object",
            variable_name,
            manifest_file.display()
        ));
    };

    manifest.insert(
        "line_number".to_string(),
        line_numbers
            .get(&variable_name)
            .map_or(Value::Null, |line_number| Value::from(*line_number)),
    );
    manifest.insert("variable_name".to_string(), Value::String(variable_name));

    Ok(Value::Object(manifest))
}
/// Pretty-print any serializable value.
fn format_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|error| format!("Failed to format JSON: {error}"))
}
