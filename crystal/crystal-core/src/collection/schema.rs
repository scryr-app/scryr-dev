use serde_json::Value;

const KEYWORDS: &[&str] = &[
    "$defs",
    "$ref",
    "title",
    "description",
    "default",
    "type",
    "properties",
    "required",
    "additionalProperties",
    "items",
    "enum",
    "const",
    "anyOf",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "minLength",
    "maxLength",
    "minItems",
    "maxItems",
];

pub(super) fn validate_schema(root: &Value) -> Result<(), String> {
    inspect(root, root, &mut Vec::new(), 0)
}
fn inspect(
    schema: &Value,
    root: &Value,
    refs: &mut Vec<String>,
    depth: usize,
) -> Result<(), String> {
    if depth > 64 {
        return Err("schema nesting exceeds 64 levels".into());
    }
    let object = schema.as_object().ok_or("schema must be an object")?;
    for key in object.keys() {
        if !KEYWORDS.contains(&key.as_str()) {
            return Err(format!("unsupported output schema keyword: {key}"));
        }
    }
    if let Some(reference) = schema.get("$ref") {
        let reference = reference.as_str().ok_or("$ref must be a string")?;
        if refs.iter().any(|item| item == reference) {
            return Err("recursive output schemas are unsupported".into());
        }
        refs.push(reference.to_owned());
        inspect(resolve(root, reference)?, root, refs, depth + 1)?;
        refs.pop();
    }
    if let Some(kind) = schema.get("type")
        && !matches!(
            kind.as_str(),
            Some("object" | "array" | "string" | "integer" | "number" | "boolean" | "null")
        )
    {
        return Err("unsupported schema type".into());
    }
    for key in ["$defs", "properties"] {
        if let Some(values) = schema.get(key) {
            for child in values
                .as_object()
                .ok_or("schema properties/defs must be objects")?
                .values()
            {
                inspect(child, root, refs, depth + 1)?;
            }
        }
    }
    for key in ["items", "additionalProperties"] {
        if let Some(child) = schema.get(key)
            && (key != "additionalProperties" || !child.is_boolean())
        {
            inspect(child, root, refs, depth + 1)?;
        }
    }
    if let Some(values) = schema.get("anyOf") {
        let values = values.as_array().ok_or("anyOf must be an array")?;
        if values.is_empty() {
            return Err("anyOf must be nonempty".into());
        }
        for child in values {
            inspect(child, root, refs, depth + 1)?;
        }
    }
    if let Some(required) = schema.get("required")
        && !required
            .as_array()
            .is_some_and(|items| items.iter().all(Value::is_string))
    {
        return Err("required must be a string array".into());
    }
    if let Some(values) = schema.get("enum")
        && values.as_array().is_none_or(std::vec::Vec::is_empty)
    {
        return Err("enum must be nonempty array".into());
    }
    for key in ["minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum"] {
        if schema.get(key).is_some_and(|value| !value.is_number()) {
            return Err(format!("{key} must be numeric"));
        }
    }
    for key in ["minLength", "maxLength", "minItems", "maxItems"] {
        if schema
            .get(key)
            .is_some_and(|value| value.as_u64().is_none())
        {
            return Err(format!("{key} must be a nonnegative integer"));
        }
    }
    Ok(())
}
fn resolve<'a>(root: &'a Value, reference: &str) -> Result<&'a Value, String> {
    let name = reference
        .strip_prefix("#/$defs/")
        .filter(|name| !name.is_empty() && !name.contains('/'))
        .ok_or("only local $defs references are supported")?;
    let name = name.replace("~1", "/").replace("~0", "~");
    root.get("$defs")
        .and_then(|defs| defs.get(name))
        .ok_or_else(|| format!("unresolved schema reference {reference}"))
}
pub(super) fn validate_data(schema: &Value, value: &Value) -> Result<(), String> {
    if matches_value(schema, schema, value, 0) {
        Ok(())
    } else {
        Err("collector output does not match its declared schema".into())
    }
}
fn matches_value(schema: &Value, root: &Value, value: &Value, depth: usize) -> bool {
    if depth > 64 {
        return false;
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str)
        && !resolve(root, reference)
            .is_ok_and(|resolved| matches_value(resolved, root, value, depth + 1))
    {
        return false;
    }
    if let Some(kind) = schema.get("type").and_then(Value::as_str) {
        let matches = match kind {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "number" => value.is_number(),
            "integer" => value.is_i64() || value.is_u64(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => false,
        };
        if !matches {
            return false;
        }
    }
    if schema
        .get("const")
        .is_some_and(|expected| expected != value)
    {
        return false;
    }
    if schema
        .get("enum")
        .and_then(Value::as_array)
        .is_some_and(|options| !options.contains(value))
    {
        return false;
    }
    if schema
        .get("anyOf")
        .and_then(Value::as_array)
        .is_some_and(|options| {
            !options
                .iter()
                .any(|child| matches_value(child, root, value, depth + 1))
        })
    {
        return false;
    }
    if let Some(object) = value.as_object() {
        if schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|keys| {
                keys.iter()
                    .filter_map(Value::as_str)
                    .any(|key| !object.contains_key(key))
            })
        {
            return false;
        }
        for (key, child) in object {
            if let Some(property) = schema.get("properties").and_then(|props| props.get(key)) {
                if !matches_value(property, root, child, depth + 1) {
                    return false;
                }
            } else if let Some(extra) = schema.get("additionalProperties")
                && (extra == &Value::Bool(false)
                    || (extra.is_object() && !matches_value(extra, root, child, depth + 1)))
            {
                return false;
            }
        }
    }
    if let Some(array) = value.as_array() {
        if !within_length(schema, "minItems", "maxItems", array.len()) {
            return false;
        }
        if schema.get("items").is_some_and(|items| {
            array
                .iter()
                .any(|child| !matches_value(items, root, child, depth + 1))
        }) {
            return false;
        }
    }
    if let Some(string) = value.as_str()
        && !within_length(schema, "minLength", "maxLength", string.chars().count())
    {
        return false;
    }
    within_numeric_bounds(schema, value)
}
fn within_numeric_bounds(schema: &Value, value: &Value) -> bool {
    if !value.is_number() {
        return true;
    }
    for key in ["minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum"] {
        if let Some(bound) = schema.get(key) {
            let ordering = compare_numbers(value, bound);
            if match key {
                "minimum" => ordering.is_lt(),
                "maximum" => ordering.is_gt(),
                "exclusiveMinimum" => !ordering.is_gt(),
                _ => !ordering.is_lt(),
            } {
                return false;
            }
        }
    }
    true
}
fn integer(value: &Value) -> Option<i128> {
    value
        .as_i64()
        .map(i128::from)
        .or_else(|| value.as_u64().map(i128::from))
}
fn compare_numbers(left: &Value, right: &Value) -> std::cmp::Ordering {
    match (integer(left), integer(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(left), None) => {
            compare_float_integer(right.as_f64().unwrap_or_default(), left).reverse()
        }
        (None, Some(right)) => compare_float_integer(left.as_f64().unwrap_or_default(), right),
        (None, None) => left
            .as_f64()
            .unwrap_or_default()
            .partial_cmp(&right.as_f64().unwrap_or_default())
            .unwrap_or(std::cmp::Ordering::Equal),
    }
}
// JSON integers fit i128. Saturation for enormous floats preserves their ordering;
// truncation plus the fractional sign compares without rounding the integer to f64.
#[allow(clippy::cast_possible_truncation)]
fn compare_float_integer(float: f64, integer: i128) -> std::cmp::Ordering {
    let whole = float as i128;
    match whole.cmp(&integer) {
        std::cmp::Ordering::Equal => float
            .fract()
            .partial_cmp(&0.0)
            .unwrap_or(std::cmp::Ordering::Equal),
        ordering => ordering,
    }
}

fn within_length(schema: &Value, min: &str, max: &str, length: usize) -> bool {
    schema
        .get(min)
        .and_then(Value::as_u64)
        .is_none_or(|bound| (length as u64) >= bound)
        && schema
            .get(max)
            .and_then(Value::as_u64)
            .is_none_or(|bound| (length as u64) <= bound)
}
