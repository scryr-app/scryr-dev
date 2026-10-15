//! Helpers for extracting GraphQL fields from raw manifest JSON.

use serde_json::Value;

pub(super) fn section_value<'a>(
    raw_json: &'a Value,
    section: &str,
    key: &str,
) -> Option<&'a Value> {
    raw_json.get(section).and_then(|section| section.get(key))
}

pub(super) fn top_value<'a>(raw_json: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| raw_json.get(key))
}

pub(super) fn section_value_or_top<'a>(
    raw_json: &'a Value,
    section: &str,
    key: &str,
    top_keys: &[&str],
) -> Option<&'a Value> {
    section_value(raw_json, section, key).or_else(|| top_value(raw_json, top_keys))
}

pub(super) fn section_string_or_top(
    raw_json: &Value,
    section: &str,
    key: &str,
    top_keys: &[&str],
) -> Option<String> {
    section_value_or_top(raw_json, section, key, top_keys)
        .and_then(Value::as_str)
        .map(std::string::ToString::to_string)
}

pub(super) fn section_string_array_or_top(
    raw_json: &Value,
    section: &str,
    key: &str,
    top_keys: &[&str],
) -> Vec<String> {
    section_value_or_top(raw_json, section, key, top_keys)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(std::string::ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}
