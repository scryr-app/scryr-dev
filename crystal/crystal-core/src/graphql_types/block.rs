//! GraphQL block types backed by raw manifest JSON.

use super::json_access::{
    section_string_array_or_top, section_string_or_top, section_value_or_top,
};
use async_graphql::{Object, SimpleObject};
use serde_json::Value;

/// `ScryrComponent` represents a parsed Pydantic model from Python.
/// The actual data is stored as JSON and exposed via GraphQL fields.
pub struct Block {
    /// The raw JSON representation of the component.
    pub raw_json: Value,
}

/// External reference associated with a block (e.g., repo, docs, dashboard).
#[derive(SimpleObject, Clone)]
pub struct Link {
    /// Human-readable name for the link target.
    pub site_name: Option<String>,
    /// URL of the target resource.
    pub http_url: Option<String>,
}

#[Object]
impl Block {
    /// Stable Manifest identity used to attach operational history.
    pub async fn manifest_id(&self) -> Option<String> {
        self.raw_json
            .get("manifestId")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }

    /// Recent GitHub workflow runs and their observed status history.
    pub async fn github_actions(&self) -> Option<async_graphql::Json<Value>> {
        self.raw_json
            .get("cicd")?
            .get("githubActions")
            .filter(|value| !value.is_null())
            .cloned()
            .map(async_graphql::Json)
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

    /// Repository or source URL (also used by `GithubCard`).
    pub async fn source_code_url(&self) -> Option<String> {
        section_string_or_top(
            &self.raw_json,
            "github",
            "repoUrl",
            &["source_code_url", "sourceCodeUrl"],
        )
    }

    /// Short description of the component.
    pub async fn description(&self) -> Option<String> {
        section_string_or_top(&self.raw_json, "info", "description", &["description"])
    }

    /// Semver-like version string (e.g. "1.2.3").
    pub async fn version(&self) -> Option<String> {
        section_string_or_top(&self.raw_json, "info", "version", &["version"])
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

    /// Primary programming language.
    pub async fn language(&self) -> Option<String> {
        section_string_or_top(&self.raw_json, "info", "language", &["language"])
    }

    /// Web frameworks in use.
    pub async fn frameworks(&self) -> Vec<String> {
        self.raw_json
            .get("info")
            .and_then(|section| section.get("frameworks"))
            .or_else(|| self.raw_json.get("frameworks"))
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        // Handle both simple strings and nested arrays [language, framework].
                        item.as_str()
                            .map(std::string::ToString::to_string)
                            .or_else(|| {
                                item.as_array()
                                    .and_then(|arr| arr.get(1))
                                    .and_then(Value::as_str)
                                    .map(std::string::ToString::to_string)
                            })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Primary deployment target identifier.
    pub async fn deployment(&self) -> Option<String> {
        section_value_or_top(&self.raw_json, "info", "deployment", &["deployment"]).and_then(
            |value| {
                // Handle both string and array formats [provider, target].
                value
                    .as_str()
                    .map(std::string::ToString::to_string)
                    .or_else(|| {
                        value
                            .as_array()
                            .and_then(|arr| arr.get(1))
                            .and_then(Value::as_str)
                            .map(std::string::ToString::to_string)
                    })
            },
        )
    }

    /// Cloud provider extracted from the deployment tuple.
    pub async fn deployment_provider(&self) -> Option<String> {
        section_value_or_top(&self.raw_json, "info", "deployment", &["deployment"]).and_then(
            |value| {
                value
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(Value::as_str)
                    .map(std::string::ToString::to_string)
            },
        )
    }

    /// Team responsible for this service.
    pub async fn owner_team(&self) -> Option<String> {
        section_string_or_top(
            &self.raw_json,
            "info",
            "ownerTeam",
            &["owner_team", "ownerTeam"],
        )
    }

    /// Primary authentication mechanism.
    pub async fn auth_type(&self) -> Option<String> {
        section_string_or_top(
            &self.raw_json,
            "info",
            "authType",
            &["auth_type", "authType"],
        )
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

    /// Monitoring and metrics platform.
    pub async fn monitoring(&self) -> Option<String> {
        section_string_or_top(&self.raw_json, "info", "monitoring", &["monitoring"])
    }

    /// Log aggregation and analysis platform.
    pub async fn log_aggregation(&self) -> Option<String> {
        section_string_or_top(
            &self.raw_json,
            "info",
            "logAggregation",
            &["log_aggregation", "logAggregation"],
        )
    }

    /// Distributed tracing and APM platform.
    pub async fn tracing(&self) -> Option<String> {
        section_string_or_top(&self.raw_json, "info", "tracing", &["tracing"])
    }

    /// Documentation URLs.
    pub async fn docs(&self) -> Vec<String> {
        section_string_array_or_top(&self.raw_json, "info", "docs", &["docs"])
    }

    /// Helpful external links with display name and URL.
    pub async fn links(&self) -> Vec<Link> {
        self.raw_json
            .get("info")
            .and_then(|section| section.get("links"))
            .or_else(|| self.raw_json.get("links"))
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        item.as_object().map_or_else(
                            || {
                                item.as_str().map(|url| Link {
                                    site_name: None,
                                    http_url: Some(url.to_string()),
                                })
                            },
                            |obj| {
                                Some(Link {
                                    site_name: obj
                                        .get("site_name")
                                        .and_then(Value::as_str)
                                        .map(std::string::ToString::to_string),
                                    http_url: obj
                                        .get("http_url")
                                        .and_then(Value::as_str)
                                        .map(std::string::ToString::to_string),
                                })
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the raw JSON as a string for debugging.
    pub async fn raw_json_string(&self) -> String {
        self.raw_json.to_string()
    }

    /// CI/CD pipeline platform (`github_actions`, `jenkins`, `circleci`, etc.).
    pub async fn cicd_tool(&self) -> Option<String> {
        section_string_or_top(
            &self.raw_json,
            "cicd",
            "platform",
            &["cicd_tool", "cicdTool"],
        )
    }

    /// Named connections to other components in the graph.
    pub async fn connections(&self) -> Vec<String> {
        self.raw_json
            .get("connections")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(std::string::ToString::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Infrastructure as Code tooling (terraform, pulumi, cdk, etc.).
    pub async fn iac_tool(&self) -> Option<String> {
        section_string_or_top(&self.raw_json, "info", "iacTool", &["iac_tool", "iacTool"])
    }

    /// Maximum number of service replicas.
    pub async fn max_replicas(&self) -> Option<i32> {
        section_value_or_top(
            &self.raw_json,
            "info",
            "maxReplicas",
            &["max_replicas", "maxReplicas"],
        )
        .and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
    }

    /// Minimum number of service replicas.
    pub async fn min_replicas(&self) -> Option<i32> {
        section_value_or_top(
            &self.raw_json,
            "info",
            "minReplicas",
            &["min_replicas", "minReplicas"],
        )
        .and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
    }
}

fn classification_icon(classification: &str) -> Option<String> {
    Some(
        match classification {
            "public_api" => "🌍",
            "internal_api" => "🔌",
            "network_router" => "🛡️",
            "public_ui" => "🖥️",
            "internal_ui" => "🧰",
            "mobile_app" => "📱",
            "admin_ui" | "job_processor" => "⚙️",
            "cli" => "⌨️",
            "sdk" => "🧩",
            "datastore" => "🗃️",
            "database" => "🗄️",
            "object_storage" => "🪣",
            "document_store" => "📄",
            "vector_store" => "🧠",
            "columnar_store" => "📚",
            "cache_store" => "⚡",
            "search_store" => "🔎",
            "queue" => "📬",
            "source" => "📥",
            "sink" => "📤",
            "worker" => "👷",
            "scheduler" => "⏰",
            _ => return None,
        }
        .to_string(),
    )
}

fn block_icon(raw_json: &Value) -> Option<String> {
    let icon = raw_json
        .get("icon")
        .and_then(Value::as_str)
        .filter(|icon| !icon.trim().is_empty())
        .map(std::string::ToString::to_string);
    if icon.is_some() {
        return icon;
    }

    raw_json
        .get("classification")
        .or_else(|| raw_json.get("consumer_type"))
        .and_then(Value::as_str)
        .and_then(classification_icon)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{block_icon, classification_icon};

    #[test]
    fn block_icon_falls_back_to_classification_icon() {
        let block = json!({
            "icon": "",
            "classification": "database"
        });

        assert_eq!(block_icon(&block).as_deref(), Some("🗄️"));
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
    fn classification_icon_returns_none_for_unknown_values() {
        assert_eq!(classification_icon("unknown"), None);
    }
}
