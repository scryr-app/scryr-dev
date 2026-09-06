//! Shared scalar formatting helpers for generated text artifacts.

use super::forge::{ForgeConfigMap, ForgeValue};

/// Return sorted keys for deterministic output.
pub(super) fn sorted_keys(object: &ForgeConfigMap) -> Vec<&str> {
    let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}

/// Return a string representation for a primitive env value.
pub(super) fn env_value_string(value: &ForgeValue) -> Option<String> {
    match value {
        ForgeValue::String(text) => Some(text.clone()),
        ForgeValue::Number(number) => Some(number.to_string()),
        ForgeValue::Bool(value) => Some(value.to_string()),
        ForgeValue::Object(object) => object.get("value").and_then(env_value_string),
        _ => None,
    }
}

/// Quote a YAML scalar using JSON string escaping, which is valid YAML.
pub(super) fn yaml_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| format!("\"{value}\""))
}

/// Return a YAML mapping key, quoting only when needed.
pub(super) fn yaml_key(key: &str) -> String {
    if key
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
    {
        key.to_string()
    } else {
        yaml_string(key)
    }
}
