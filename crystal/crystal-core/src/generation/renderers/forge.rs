//! Forge selection from generated manifest envelopes.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Deterministic Forge config mapping used by renderers.
pub(super) type ForgeConfigMap = BTreeMap<String, ForgeValue>;

/// JSON-compatible Forge config value parsed once at the renderer boundary.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub(super) enum ForgeValue {
    /// JSON null.
    Null,
    /// Boolean scalar.
    Bool(bool),
    /// Numeric scalar.
    Number(serde_json::Number),
    /// String scalar.
    String(String),
    /// Array value.
    Array(Vec<Self>),
    /// Object value.
    Object(ForgeConfigMap),
}

impl ForgeValue {
    /// Return object contents when this value is an object.
    pub(super) const fn as_object(&self) -> Option<&ForgeConfigMap> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    /// Return string contents when this value is a string.
    pub(super) const fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

/// Generated manifest envelope fields needed by Forge renderers.
#[derive(Deserialize)]
struct GeneratedForgeEnvelope {
    /// Public Forge values emitted by the manifest adapter.
    forges: Vec<ForgeObject>,
}

/// Serialized Forge fields consumed by generated config renderers.
#[derive(Clone, Deserialize)]
pub(super) struct ForgeObject {
    /// Human-readable Forge name.
    pub(super) name: Option<String>,
    /// Python variable name that declared the Forge.
    pub(super) variable_name: Option<String>,
    /// Tool metadata consumed by config renderers.
    #[serde(default)]
    pub(super) tools: ForgeConfigMap,
    /// Environment variables consumed by config renderers.
    #[serde(default)]
    pub(super) env: ForgeConfigMap,
    /// Task metadata consumed by config renderers.
    #[serde(default)]
    pub(super) tasks: ForgeConfigMap,
    /// Remaining serialized Forge fields preserved for mise rendering.
    #[serde(flatten)]
    extra: ForgeConfigMap,
}

impl ForgeObject {
    /// Rebuild a JSON object for renderers that need arbitrary Forge keys.
    pub(super) fn to_mise_source_object(&self) -> ForgeConfigMap {
        let mut object = self.extra.clone();
        if !self.tools.is_empty() {
            object.insert("tools".to_string(), ForgeValue::Object(self.tools.clone()));
        }
        if !self.env.is_empty() {
            object.insert("env".to_string(), ForgeValue::Object(self.env.clone()));
        }
        if !self.tasks.is_empty() {
            object.insert("tasks".to_string(), ForgeValue::Object(self.tasks.clone()));
        }
        object
    }
}

/// Return the selected Forge object from a generated manifest envelope.
pub(super) fn selected_forge(
    manifest_json: &str,
    forge_selector: Option<&str>,
) -> Result<ForgeObject, String> {
    let envelope = serde_json::from_str::<GeneratedForgeEnvelope>(manifest_json)
        .map_err(|error| format!("Generated manifest JSON was malformed: {error}"))?;

    select_forge(&envelope.forges, forge_selector).cloned()
}

/// Return a readable forge name from a serialized Forge object.
pub(super) fn forge_display_name(forge: &ForgeObject) -> Option<String> {
    match (forge.name.as_deref(), forge.variable_name.as_deref()) {
        (Some(name), Some(variable)) => Some(format!("{name} ({variable})")),
        (Some(name), None) => Some(name.to_string()),
        (None, Some(variable)) => Some(variable.to_string()),
        (None, None) => None,
    }
}

/// Select exactly one forge by optional name or variable.
fn select_forge<'a>(
    forges: &'a [ForgeObject],
    forge_selector: Option<&str>,
) -> Result<&'a ForgeObject, String> {
    if forges.is_empty() {
        return Err("Manifest file did not define any public Forge instances".to_string());
    }

    if let Some(selector) = forge_selector {
        let matches = forges
            .iter()
            .filter(|forge| forge_matches_selector(forge, selector))
            .collect::<Vec<_>>();
        return match matches.as_slice() {
            [forge] => Ok(forge),
            [] => Err(format!("No Forge matched `{selector}`")),
            _ => Err(format!("Multiple Forges matched `{selector}`")),
        };
    }

    if let [forge] = forges {
        Ok(forge)
    } else {
        let names = forges
            .iter()
            .filter_map(forge_display_name)
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!(
            "Manifest file defined multiple Forges. Pass `--forge <name-or-variable>`. Available: {names}"
        ))
    }
}

/// Return whether a forge matches a CLI selector.
fn forge_matches_selector(forge: &ForgeObject, selector: &str) -> bool {
    forge
        .variable_name
        .as_deref()
        .is_some_and(|value| value == selector)
        || forge.name.as_deref().is_some_and(|value| value == selector)
}
