//! Docker Compose renderer for Forge tools.

use super::forge::{ForgeObject, ForgeValue, forge_display_name, selected_forge};
use super::scalar::{sorted_keys, yaml_key, yaml_string};
use std::collections::BTreeMap;

/// Render Docker Compose YAML from known service tools in the selected Forge.
pub(crate) fn render_compose_yaml(
    manifest_json: &str,
    forge_selector: Option<&str>,
) -> Result<String, String> {
    let forge = selected_forge(manifest_json, forge_selector)?;
    let services = compose_services_for_forge(&forge);
    if services.is_empty() {
        let name = forge_display_name(&forge).unwrap_or_else(|| "selected Forge".to_string());
        return Err(format!(
            "{name} does not define any Docker Compose service tools. Supported tools: postgres, postgresql, mysql, mariadb, mongodb, redis"
        ));
    }

    let mut lines = vec!["services:".to_string()];
    for service in &services {
        lines.push(format!("  {}:", service.name));
        lines.push(format!("    image: {}", yaml_string(&service.image)));
        if !service.environment.is_empty() {
            lines.push("    environment:".to_string());
            for key in service.environment.keys() {
                let value = service
                    .environment
                    .get(key)
                    .ok_or_else(|| format!("Missing expected environment key `{key}`"))?;
                lines.push(format!("      {}: {}", yaml_key(key), yaml_string(value)));
            }
        }
        if !service.ports.is_empty() {
            lines.push("    ports:".to_string());
            for port in &service.ports {
                lines.push(format!("      - {}", yaml_string(port)));
            }
        }
        if let Some(volume) = &service.volume {
            lines.push("    volumes:".to_string());
            let mount = format!("{}:{}", volume.name, volume.mount_path);
            lines.push(format!("      - {}", yaml_string(&mount)));
        }
    }

    let volumes = services
        .iter()
        .filter_map(|service| service.volume.as_ref())
        .collect::<Vec<_>>();
    if !volumes.is_empty() {
        lines.push(String::new());
        lines.push("volumes:".to_string());
        for volume in volumes {
            lines.push(format!("  {}:", volume.name));
        }
    }

    Ok(format!("{}\n", lines.join("\n")))
}

/// One Docker Compose service inferred from a Forge tool.
struct ComposeService {
    /// Compose service name.
    name: String,
    /// Docker image reference.
    image: String,
    /// Published ports.
    ports: Vec<String>,
    /// Service environment variables.
    environment: BTreeMap<String, String>,
    /// Optional named volume.
    volume: Option<ComposeVolume>,
}

/// One named Docker Compose volume mount.
struct ComposeVolume {
    /// Volume name.
    name: String,
    /// Container mount path.
    mount_path: String,
}

/// Return Compose services for known infrastructure tools in a Forge.
fn compose_services_for_forge(forge: &ForgeObject) -> Vec<ComposeService> {
    let mut services = Vec::new();
    for key in sorted_keys(&forge.tools) {
        let Some(spec) = forge.tools.get(key) else {
            continue;
        };
        let version = tool_version(spec);
        if let Some(service) = compose_service_for_tool(key, version.as_deref()) {
            services.push(service);
        }
    }
    services
}

/// Return a Compose service for one supported tool.
fn compose_service_for_tool(tool: &str, version: Option<&str>) -> Option<ComposeService> {
    let normalized = tool.replace('-', "_").to_ascii_lowercase();
    match normalized.as_str() {
        "postgres" | "postgresql" => Some(ComposeService {
            name: "postgres".to_string(),
            image: image_with_tag("postgres", version),
            ports: vec!["5432:5432".to_string()],
            environment: BTreeMap::from_iter([
                ("POSTGRES_DB".to_string(), "postgres".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "postgres".to_string()),
                ("POSTGRES_USER".to_string(), "postgres".to_string()),
            ]),
            volume: Some(ComposeVolume {
                name: "postgres-data".to_string(),
                mount_path: "/var/lib/postgresql/data".to_string(),
            }),
        }),
        "mongodb" | "mongo" => Some(ComposeService {
            name: "mongodb".to_string(),
            image: image_with_tag("mongo", version),
            ports: vec!["27017:27017".to_string()],
            environment: BTreeMap::new(),
            volume: Some(ComposeVolume {
                name: "mongodb-data".to_string(),
                mount_path: "/data/db".to_string(),
            }),
        }),
        "redis" => Some(ComposeService {
            name: "redis".to_string(),
            image: image_with_tag("redis", version),
            ports: vec!["6379:6379".to_string()],
            environment: BTreeMap::new(),
            volume: None,
        }),
        "mysql" => Some(ComposeService {
            name: "mysql".to_string(),
            image: image_with_tag("mysql", version),
            ports: vec!["3306:3306".to_string()],
            environment: BTreeMap::from_iter([
                ("MYSQL_DATABASE".to_string(), "mysql".to_string()),
                ("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string()),
            ]),
            volume: Some(ComposeVolume {
                name: "mysql-data".to_string(),
                mount_path: "/var/lib/mysql".to_string(),
            }),
        }),
        "mariadb" => Some(ComposeService {
            name: "mariadb".to_string(),
            image: image_with_tag("mariadb", version),
            ports: vec!["3306:3306".to_string()],
            environment: BTreeMap::from_iter([
                ("MARIADB_DATABASE".to_string(), "mariadb".to_string()),
                ("MARIADB_ROOT_PASSWORD".to_string(), "mariadb".to_string()),
            ]),
            volume: Some(ComposeVolume {
                name: "mariadb-data".to_string(),
                mount_path: "/var/lib/mysql".to_string(),
            }),
        }),
        _ => None,
    }
}

/// Return the tool version from any supported Forge tool spec shape.
fn tool_version(spec: &ForgeValue) -> Option<String> {
    match spec {
        ForgeValue::String(version) => Some(version.clone()),
        ForgeValue::Array(versions) => versions
            .first()
            .and_then(ForgeValue::as_str)
            .map(ToOwned::to_owned),
        ForgeValue::Object(object) => match object.get("version") {
            Some(ForgeValue::String(version)) => Some(version.clone()),
            Some(ForgeValue::Array(versions)) => versions
                .first()
                .and_then(ForgeValue::as_str)
                .map(ToOwned::to_owned),
            _ => None,
        },
        _ => None,
    }
}

/// Return an image reference with a stable default tag.
fn image_with_tag(image: &str, version: Option<&str>) -> String {
    format!("{}:{}", image, version.unwrap_or("latest"))
}

#[cfg(test)]
mod tests {
    use super::render_compose_yaml;
    use serde_json::json;

    #[test]
    fn compose_renderer_emits_services_for_string_and_detailed_tools() -> Result<(), String> {
        let manifest_json = json!({
            "files": [],
            "manifests": [],
            "forges": [{
                "name": "Local",
                "variable_name": "local_forge",
                "tools": {
                    "mongodb": "8",
                    "postgres": {"version": "16"},
                    "redis": ["7"]
                }
            }],
            "diagrams": []
        })
        .to_string();

        let compose = render_compose_yaml(&manifest_json, None)?;

        assert!(compose.contains("mongodb:"));
        assert!(compose.contains("image: \"mongo:8\""));
        assert!(compose.contains("postgres:"));
        assert!(compose.contains("image: \"postgres:16\""));
        assert!(compose.contains("redis:"));
        assert!(compose.contains("image: \"redis:7\""));
        assert!(compose.contains("postgres-data:"));
        Ok(())
    }

    #[test]
    fn compose_renderer_requires_a_matching_forge() {
        let manifest_json = json!({
            "files": [],
            "manifests": [],
            "forges": [],
            "diagrams": []
        })
        .to_string();

        let error = render_compose_yaml(&manifest_json, None).err();

        assert_eq!(
            error.as_deref(),
            Some("Manifest file did not define any public Forge instances")
        );
    }
}
