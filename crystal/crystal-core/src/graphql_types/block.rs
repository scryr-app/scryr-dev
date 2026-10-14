//! GraphQL block types backed by raw manifest JSON.

use async_graphql::Object;
use serde_json::Value;

/// Architecture component represented by manifest JSON.
pub struct Block {
    /// Raw architecture component data.
    pub raw_json: Value,
}

#[Object]
impl Block {
    /// Stable identity for the architecture component.
    pub async fn manifest_id(&self) -> Option<String> {
        self.raw_json
            .get("manifestId")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }

    /// Display name for the component (used as the block label).
    pub async fn name(&self) -> Option<String> {
        self.raw_json
            .get("name")
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string)
    }

    /// Emoji or small icon string for the component.
    pub async fn icon(&self) -> Option<String> {
        block_icon(&self.raw_json)
    }

    /// 1-based line number of the manifest assignment in the source file.
    pub async fn line_number(&self) -> Option<i32> {
        self.raw_json
            .get("line_number")
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
    }

    /// Service classification (e.g. `public_api`, `worker`, `database`).
    pub async fn consumer_type(&self) -> Option<String> {
        self.raw_json
            .get("classification")
            .or_else(|| self.raw_json.get("consumer_type"))
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string)
    }

    /// Arbitrary tags for categorization and filtering.
    pub async fn tags(&self) -> Vec<String> {
        self.raw_json
            .get("tags")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the raw JSON as a string for debugging.
    pub async fn raw_json_string(&self) -> String {
        self.raw_json.to_string()
    }

    /// Named connections to other components in the graph.
    pub async fn connections(&self) -> Vec<String> {
        block_connections(&self.raw_json)
    }
}

fn block_icon(raw_json: &Value) -> Option<String> {
    raw_json
        .get("icon")
        .and_then(Value::as_str)
        .filter(|icon| !icon.trim().is_empty())
        .map(str::to_owned)
}

fn block_connections(raw_json: &Value) -> Vec<String> {
    raw_json
        .get("connections")
        .and_then(Value::as_array)
        .map(|connections| {
            connections
                .iter()
                .filter_map(|connection| {
                    connection
                        .as_str()
                        .or_else(|| connection.get("name").and_then(Value::as_str))
                        .filter(|name| !name.trim().is_empty())
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{block_connections, block_icon};

    #[test]
    fn block_icon_does_not_invent_classification_icon() {
        let block = json!({
            "icon": "",
            "classification": "database"
        });

        assert_eq!(block_icon(&block), None);
    }

    #[test]
    fn block_icon_prefers_explicit_icon() {
        let block = json!({
            "icon": "🧪",
            "classification": "database"
        });

        assert_eq!(block_icon(&block).as_deref(), Some("🧪"));
    }

    #[test]
    fn block_connections_support_manifest_references_and_legacy_names() {
        let block = json!({
            "connections": [
                { "name": "Database", "connections": [] },
                "Legacy Queue"
            ]
        });

        assert_eq!(
            block_connections(&block),
            vec!["Database".to_string(), "Legacy Queue".to_string()]
        );
    }

    #[test]
    fn block_connections_ignore_invalid_or_empty_references() {
        let block = json!({
            "connections": [{ "name": "" }, {}, null, 42]
        });

        assert!(block_connections(&block).is_empty());
    }

    #[test]
    fn missing_icon_stays_absent() {
        assert_eq!(block_icon(&json!({"name": "API"})), None);
    }
}
