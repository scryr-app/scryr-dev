//! GraphQL block types backed by raw manifest JSON.

use super::json_access::{section_string_array_or_top, section_string_or_top};
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

    /// Programming languages used by the component.
    pub async fn languages(&self) -> Vec<String> {
        block_languages(&self.raw_json)
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

    /// Hosting targets and Infrastructure as Code tools used for deployment.
    pub async fn deployments(&self) -> Vec<String> {
        block_deployments(&self.raw_json)
    }

    /// Owners responsible for this service.
    pub async fn owners(&self) -> Option<String> {
        block_owners(&self.raw_json)
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

    /// Monitoring, tracing, and log aggregation tools.
    pub async fn metric_tools(&self) -> Vec<String> {
        block_metric_tools(&self.raw_json)
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
        block_connections(&self.raw_json)
    }
}

// Accept old persisted manifests at the read boundary. New list fields take
// precedence even when empty, so intentionally cleared values never reappear.
fn info_value<'a>(raw_json: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter()
        .find_map(|key| raw_json.get("info").and_then(|info| info.get(key)))
        .or_else(|| keys.iter().find_map(|key| raw_json.get(key)))
}

fn block_owners(raw_json: &Value) -> Option<String> {
    info_value(raw_json, &["owners"])
        .or_else(|| info_value(raw_json, &["ownerTeam", "owner_team"]))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn unique_tools<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        let value = match value.trim() {
            "" | "none" => continue,
            "sumologic" => "sumo_logic",
            other => other,
        };
        if !result.iter().any(|existing| existing == value) {
            result.push(value.to_owned());
        }
    }
    result
}

fn string_list(value: &Value) -> Vec<String> {
    unique_tools(
        value
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str),
    )
}

fn block_languages(raw_json: &Value) -> Vec<String> {
    if let Some(value) = info_value(raw_json, &["languages"]) {
        return string_list(value);
    }
    unique_tools(
        info_value(raw_json, &["language"])
            .and_then(Value::as_str)
            .into_iter(),
    )
}

fn block_deployments(raw_json: &Value) -> Vec<String> {
    if let Some(value) = info_value(raw_json, &["deployments"]) {
        return string_list(value);
    }
    let target = info_value(raw_json, &["deployment"]).and_then(|value| {
        value.as_str().or_else(|| {
            let tuple = value.as_array()?;
            let target = tuple.get(1)?.as_str()?;
            if tuple.first().and_then(Value::as_str) == Some("azure") && target == "batch" {
                Some("batch_azure")
            } else {
                Some(target)
            }
        })
    });
    let iac = info_value(raw_json, &["iacTool", "iac_tool"]).and_then(Value::as_str);
    unique_tools(target.into_iter().chain(iac))
}

fn block_metric_tools(raw_json: &Value) -> Vec<String> {
    if let Some(value) = info_value(raw_json, &["metricTools", "metric_tools"]) {
        return string_list(value);
    }
    unique_tools(
        [
            &["monitoring"][..],
            &["tracing"][..],
            &["logAggregation", "log_aggregation"][..],
        ]
        .into_iter()
        .filter_map(|keys| info_value(raw_json, keys).and_then(Value::as_str)),
    )
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
    use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
    use serde_json::json;

    use super::{
        Block, block_connections, block_deployments, block_icon, block_languages,
        block_metric_tools, block_owners,
    };

    struct InfoQuery;

    #[Object]
    impl InfoQuery {
        async fn block(&self) -> Block {
            Block {
                raw_json: json!({"info": {
                    "languages": ["python", "rust"],
                    "deployments": ["ec2", "terraform"],
                    "metricTools": ["datadog", "jaeger"], "owners": "Platform and Data"
                }}),
            }
        }
    }

    #[tokio::test]
    async fn graphql_exposes_plural_lists_and_removes_obsolete_fields()
    -> Result<(), serde_json::Error> {
        let schema = Schema::build(InfoQuery, EmptyMutation, EmptySubscription).finish();
        let response = schema
            .execute("{ block { languages deployments metricTools owners } }")
            .await;
        assert_eq!(response.errors, Vec::new());
        assert_eq!(
            response.data.into_json()?,
            json!({"block": {
                "languages": ["python", "rust"],
                "deployments": ["ec2", "terraform"],
                "metricTools": ["datadog", "jaeger"], "owners": "Platform and Data"
            }})
        );
        for field in [
            "language",
            "deployment",
            "deploymentProvider",
            "authType",
            "ownerTeam",
            "monitoring",
            "tracing",
            "logAggregation",
            "iacTool",
        ] {
            let response = schema.execute(format!("{{ block {{ {field} }} }}")).await;
            assert!(
                !response.errors.is_empty(),
                "obsolete field {field} remains"
            );
        }
        Ok(())
    }

    #[test]
    fn ownership_migrates_legacy_strings_without_overriding_explicit_new_values() {
        for block in [
            json!({"info": {"ownerTeam": "Platform"}}),
            json!({"owner_team": "Platform"}),
            json!({"info": {"owners": "Platform"}}),
        ] {
            assert_eq!(block_owners(&block).as_deref(), Some("Platform"));
        }
        assert_eq!(
            block_owners(&json!({"info": {"owners": "", "ownerTeam": "Platform"}})).as_deref(),
            Some("")
        );
        assert_eq!(block_owners(&json!({})), None);
    }

    #[test]
    fn plural_info_fields_return_all_values_and_deduplicate_tools() {
        let block = json!({"info": {
            "languages": ["python", "rust"],
            "deployments": ["ec2", "terraform", "ec2"],
            "metricTools": ["datadog", "loki", "datadog", "sumo_logic"]
        }});
        assert_eq!(block_languages(&block), vec!["python", "rust"]);
        assert_eq!(block_deployments(&block), vec!["ec2", "terraform"]);
        assert_eq!(
            block_metric_tools(&block),
            vec!["datadog", "loki", "sumo_logic"]
        );
    }

    #[test]
    fn legacy_nested_info_fields_remain_readable_without_authentication() {
        let block = json!({"info": {
            "language": "javascript", "deployment": ["aws", "ec2"],
            "iacTool": "terraform", "monitoring": "datadog",
            "tracing": "datadog", "logAggregation": "sumologic", "authType": "jwt"
        }});
        assert_eq!(block_languages(&block), vec!["javascript"]);
        assert_eq!(block_deployments(&block), vec!["ec2", "terraform"]);
        assert_eq!(block_metric_tools(&block), vec!["datadog", "sumo_logic"]);
    }

    #[test]
    fn legacy_top_level_fields_and_snake_case_aliases_remain_readable() {
        let block = json!({
            "language": "python", "deployment": "helm", "iac_tool": "helm",
            "monitoring": "none", "tracing": "jaeger", "log_aggregation": "loki"
        });
        assert_eq!(block_languages(&block), vec!["python"]);
        assert_eq!(block_deployments(&block), vec!["helm"]);
        assert_eq!(block_metric_tools(&block), vec!["jaeger", "loki"]);
        assert_eq!(
            block_deployments(&json!({"deployment": ["azure", "batch"]})),
            vec!["batch_azure"]
        );
        assert_eq!(
            block_metric_tools(&json!({"info": {"metric_tools": ["grafana"]}})),
            vec!["grafana"]
        );
    }

    #[test]
    fn explicit_empty_lists_override_all_legacy_values() {
        let block = json!({
            "languages": ["ruby"], "deployments": ["heroku"], "metricTools": ["loki"],
            "info": {
                "languages": [], "language": "python",
                "deployments": [], "deployment": "ec2", "iacTool": "terraform",
                "metricTools": [], "monitoring": "datadog", "tracing": "jaeger"
            }
        });
        assert_eq!(block_languages(&block), Vec::<String>::new());
        assert_eq!(block_deployments(&block), Vec::<String>::new());
        assert_eq!(block_metric_tools(&block), Vec::<String>::new());
    }

    #[test]
    fn missing_and_malformed_info_lists_do_not_invent_defaults() {
        for block in [
            json!({}),
            json!({"info": {
                "languages": [null, 1, ""], "deployments": "ec2", "metricTools": ["none", {}]
            }}),
        ] {
            assert_eq!(block_languages(&block), Vec::<String>::new());
            assert_eq!(block_deployments(&block), Vec::<String>::new());
            assert_eq!(block_metric_tools(&block), Vec::<String>::new());
        }
    }

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

        assert_eq!(block_connections(&block), Vec::<String>::new());
    }

    #[test]
    fn missing_icon_stays_absent() {
        assert_eq!(block_icon(&json!({"name": "API"})), None);
    }
}
