//! mise.toml renderer for Forge metadata.

use super::forge::{ForgeConfigMap, ForgeObject, ForgeValue, selected_forge};
use super::scalar::sorted_keys;

/// Render a selected forge from the generated manifest envelope as mise.toml.
pub(crate) fn render_mise_toml(
    manifest_json: &str,
    forge_selector: Option<&str>,
) -> Result<String, String> {
    let forge_object = selected_forge(manifest_json, forge_selector)?;
    let mise_object = forge_to_mise_object(&forge_object);
    toml_from_object(&mise_object)
}

/// Convert a serialized Scryr Forge wrapper into a mise.toml object.
fn forge_to_mise_object(forge: &ForgeObject) -> ForgeConfigMap {
    let mut output = ForgeConfigMap::new();
    for (key, value) in forge.to_mise_source_object() {
        if matches!(
            key.as_str(),
            "name" | "description" | "file_name" | "line_number" | "variable_name"
        ) {
            continue;
        }
        if key == "raw" {
            if let Some(raw) = value.as_object() {
                for (raw_key, raw_value) in raw {
                    if let Some(value) = prune_mise_value(raw_value, None) {
                        output.insert(raw_key.clone(), value);
                    }
                }
            }
            continue;
        }
        if let Some(value) = prune_mise_value(&value, Some(&key)) {
            output.insert(key, value);
        }
    }
    output
}

/// Remove empty/default serialization noise from a value before TOML rendering.
fn prune_mise_value(value: &ForgeValue, key: Option<&str>) -> Option<ForgeValue> {
    match value {
        ForgeValue::Null => None,
        ForgeValue::String(text) if text.is_empty() && key == Some("description") => None,
        ForgeValue::Array(items) => {
            let items = items
                .iter()
                .filter_map(|item| prune_mise_value(item, None))
                .collect::<Vec<_>>();
            (!items.is_empty()).then_some(ForgeValue::Array(items))
        }
        ForgeValue::Object(object) => {
            let mut output = ForgeConfigMap::new();
            for (child_key, child_value) in object {
                if is_false_task_default(child_key, child_value) {
                    continue;
                }
                if let Some(value) = prune_mise_value(child_value, Some(child_key)) {
                    output.insert(child_key.clone(), value);
                }
            }
            (!output.is_empty()).then_some(ForgeValue::Object(output))
        }
        value => Some(value.clone()),
    }
}

/// Return whether a false field is one of the default task booleans.
fn is_false_task_default(key: &str, value: &ForgeValue) -> bool {
    value == &ForgeValue::Bool(false)
        && matches!(
            key,
            "hide" | "raw" | "raw_args" | "interactive" | "quiet" | "silent" | "tools" | "redact"
        )
}

/// Render a JSON object as TOML.
fn toml_from_object(object: &ForgeConfigMap) -> Result<String, String> {
    let mut lines = Vec::new();

    for key in sorted_keys(object) {
        let value = object
            .get(key)
            .ok_or_else(|| format!("Missing expected TOML key `{key}`"))?;
        if value.as_object().is_none() {
            lines.push(format!("{} = {}", toml_key(key), toml_value(value)?));
        }
    }

    for section in ["tools", "env", "vars", "settings", "plugins"] {
        if let Some(section_object) = object.get(section).and_then(ForgeValue::as_object) {
            push_blank_line(&mut lines);
            render_table(&mut lines, &[section], section_object)?;
        }
    }

    if let Some(tasks) = object.get("tasks").and_then(ForgeValue::as_object) {
        push_blank_line(&mut lines);
        render_tasks(&mut lines, tasks)?;
    }

    for key in sorted_keys(object) {
        if matches!(
            key,
            "tools" | "env" | "vars" | "settings" | "plugins" | "tasks"
        ) {
            continue;
        }
        if let Some(section_object) = object.get(key).and_then(ForgeValue::as_object) {
            push_blank_line(&mut lines);
            render_table(&mut lines, &[key], section_object)?;
        }
    }

    if lines.is_empty() {
        return Ok(String::new());
    }
    Ok(format!("{}\n", lines.join("\n")))
}

/// Render a normal TOML table.
fn render_table(
    lines: &mut Vec<String>,
    path: &[&str],
    object: &ForgeConfigMap,
) -> Result<(), String> {
    lines.push(toml_table_header(path));
    for key in sorted_keys(object) {
        let value = object
            .get(key)
            .ok_or_else(|| format!("Missing expected TOML key `{key}`"))?;
        lines.push(format!("{} = {}", toml_key(key), toml_value(value)?));
    }
    Ok(())
}

/// Render mise task shorthand and detailed task tables.
fn render_tasks(lines: &mut Vec<String>, tasks: &ForgeConfigMap) -> Result<(), String> {
    let mut shorthand = Vec::new();
    let mut detailed = Vec::new();

    for key in sorted_keys(tasks) {
        let value = tasks
            .get(key)
            .ok_or_else(|| format!("Missing expected task `{key}`"))?;
        if value.as_object().is_some() {
            detailed.push((key, value));
        } else {
            shorthand.push((key, value));
        }
    }

    if !shorthand.is_empty() {
        lines.push("[tasks]".to_string());
        for (key, value) in shorthand {
            lines.push(format!("{} = {}", toml_key(key), toml_value(value)?));
        }
    }

    for (key, value) in detailed {
        let Some(task) = value.as_object() else {
            continue;
        };
        push_blank_line(lines);
        lines.push(toml_table_header(&["tasks", key]));
        for task_key in sorted_keys(task) {
            let task_value = task
                .get(task_key)
                .ok_or_else(|| format!("Missing expected task key `{task_key}`"))?;
            lines.push(format!(
                "{} = {}",
                toml_key(task_key),
                toml_value(task_value)?
            ));
        }
    }

    Ok(())
}

/// Render any TOML value supported by Forge output.
fn toml_value(value: &ForgeValue) -> Result<String, String> {
    match value {
        ForgeValue::Null => Err("Cannot render null as TOML".to_string()),
        ForgeValue::Bool(value) => Ok(value.to_string()),
        ForgeValue::Number(value) => Ok(value.to_string()),
        ForgeValue::String(value) => serde_json::to_string(value)
            .map_err(|error| format!("Failed to quote TOML string: {error}")),
        ForgeValue::Array(values) => {
            let rendered = values
                .iter()
                .map(toml_value)
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            Ok(format!("[{rendered}]"))
        }
        ForgeValue::Object(object) => {
            let rendered = sorted_keys(object)
                .into_iter()
                .map(|key| {
                    let value = object
                        .get(key)
                        .ok_or_else(|| format!("Missing expected TOML key `{key}`"))?;
                    Ok(format!("{} = {}", toml_key(key), toml_value(value)?))
                })
                .collect::<Result<Vec<_>, String>>()?
                .join(", ");
            Ok(format!("{{ {rendered} }}"))
        }
    }
}

/// Return TOML table header text for a dotted path.
fn toml_table_header(path: &[&str]) -> String {
    let path = path.iter().map(|part| toml_key(part)).collect::<Vec<_>>();
    format!("[{}]", path.join("."))
}

/// Return a TOML key, quoting only when needed.
fn toml_key(key: &str) -> String {
    if key
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
    {
        key.to_string()
    } else {
        serde_json::to_string(key).unwrap_or_else(|_| format!("\"{key}\""))
    }
}

/// Separate TOML sections with one blank line.
fn push_blank_line(lines: &mut Vec<String>) {
    if !lines.is_empty() && lines.last().is_some_and(|line| !line.is_empty()) {
        lines.push(String::new());
    }
}
