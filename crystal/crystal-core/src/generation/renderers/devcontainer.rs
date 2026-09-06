//! Devcontainer renderer for Forge metadata.

use super::forge::{ForgeObject, selected_forge};
use super::scalar::{env_value_string, sorted_keys};
use serde::Serialize;
use std::collections::BTreeMap;

/// Render a devcontainer.json from the selected Forge.
pub(crate) fn render_devcontainer_json(
    manifest_json: &str,
    forge_selector: Option<&str>,
) -> Result<String, String> {
    let forge = selected_forge(manifest_json, forge_selector)?;
    let name = forge
        .name
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or("Scryr Devcontainer");
    let install_command = if forge.tasks.contains_key("install") {
        "curl https://mise.run | sh && ~/.local/bin/mise install && ~/.local/bin/mise run install"
    } else {
        "curl https://mise.run | sh && ~/.local/bin/mise install"
    };
    let container_env = string_env_object(&forge.env);
    let container_env = (!container_env.is_empty()).then_some(container_env);
    let devcontainer = Devcontainer {
        name: name.to_string(),
        image: "mcr.microsoft.com/devcontainers/base:ubuntu".to_string(),
        workspace_folder: "/workspaces/${localWorkspaceFolderBasename}".to_string(),
        post_create_command: install_command.to_string(),
        container_env,
        customizations: DevcontainerCustomizations {
            vscode: VscodeCustomizations {
                extensions: vscode_extensions_for_tools(&forge),
            },
        },
    };

    serde_json::to_string_pretty(&devcontainer)
        .map(|json| format!("{json}\n"))
        .map_err(|error| format!("Failed to serialize devcontainer.json: {error}"))
}

/// Minimal devcontainer.json emitted from a Forge.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Devcontainer {
    /// Display name.
    name: String,
    /// Base image.
    image: String,
    /// Workspace mount path.
    workspace_folder: String,
    /// Setup command.
    post_create_command: String,
    /// Environment variables exposed in the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    container_env: Option<BTreeMap<String, String>>,
    /// Editor-specific customizations.
    customizations: DevcontainerCustomizations,
}

/// Editor customization wrapper.
#[derive(Serialize)]
struct DevcontainerCustomizations {
    /// VS Code customizations.
    vscode: VscodeCustomizations,
}

/// VS Code customization block.
#[derive(Serialize)]
struct VscodeCustomizations {
    /// Extension identifiers.
    extensions: Vec<String>,
}

/// Convert Forge env values into devcontainer string environment values.
fn string_env_object(env: &super::forge::ForgeConfigMap) -> BTreeMap<String, String> {
    let mut output = BTreeMap::new();
    for key in sorted_keys(env) {
        let Some(value) = env.get(key) else {
            continue;
        };
        if let Some(text) = env_value_string(value) {
            output.insert(key.to_string(), text);
        }
    }
    output
}

/// Return VS Code extensions that match known Forge tools.
fn vscode_extensions_for_tools(forge: &ForgeObject) -> Vec<String> {
    let mut extensions = Vec::new();
    for key in sorted_keys(&forge.tools) {
        match key.replace('-', "_").to_ascii_lowercase().as_str() {
            "node" => extensions.push("dbaeumer.vscode-eslint".to_string()),
            "python" => extensions.push("ms-python.python".to_string()),
            "rust" => extensions.push("rust-lang.rust-analyzer".to_string()),
            "terraform" => extensions.push("hashicorp.terraform".to_string()),
            _ => {}
        }
    }
    extensions
}

#[cfg(test)]
mod tests {
    use super::render_devcontainer_json;
    use serde_json::{Value, json};

    #[test]
    fn devcontainer_renderer_emits_valid_json_for_forge() -> Result<(), String> {
        let manifest_json = json!({
            "files": [],
            "manifests": [],
            "forges": [{
                "name": "Node Forge",
                "variable_name": "node_forge",
                "tools": {"node": {"version": "22"}},
                "env": {"NODE_ENV": "development"},
                "tasks": {"install": {"run": "npm install"}}
            }],
            "diagrams": []
        })
        .to_string();

        let output = render_devcontainer_json(&manifest_json, Some("node_forge"))?;
        let parsed = serde_json::from_str::<Value>(&output)
            .map_err(|error| format!("devcontainer should be JSON: {error}"))?;

        assert_eq!(parsed["name"], Value::String("Node Forge".to_string()));
        assert_eq!(
            parsed["containerEnv"]["NODE_ENV"],
            Value::String("development".to_string())
        );
        assert!(
            parsed["postCreateCommand"]
                .as_str()
                .is_some_and(|command| command.contains("mise run install"))
        );
        assert_eq!(
            parsed["customizations"]["vscode"]["extensions"][0],
            Value::String("dbaeumer.vscode-eslint".to_string())
        );
        Ok(())
    }
}
