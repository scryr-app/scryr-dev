//! Explicit JSON-to-TOML migration; normal operation reads TOML exclusively.
use crate::args::MigrateSecretsArgs;
use serde::Deserialize;
use std::{collections::BTreeMap, io::Write};

/// The former connection file schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyConnection {
    /// Previously approved destination.
    endpoint: String,
    /// Optional basic authentication username.
    #[serde(default)]
    username: String,
    /// Project identifies a legacy aggregate connection.
    #[serde(default, alias = "projectId")]
    project_id: Option<u64>,
    /// Existing credential, preserved without printing it.
    token: String,
}
/// Convert the old organization map into credential tables and operator approvals.
fn convert(source: &[u8]) -> Result<String, String> {
    let organizations: BTreeMap<String, BTreeMap<String, LegacyConnection>> =
        serde_json::from_slice(source).map_err(|_| "Invalid legacy JSON connection file")?;
    let mut output = vec![String::from(
        "# Migrated integration credentials. Keep this file out of version control.\n",
    )];
    for (org, connections) in organizations {
        let prefix = if org == "local-dev-org" {
            String::new()
        } else {
            format!("organizations.{}.", quote(&org)?)
        };
        for (name, connection) in connections {
            let name = quote(&name)?;
            output.push(format!("\n[{prefix}authentication.{name}]\n"));
            if connection.project_id.is_some() {
                output.push(format!("api_key = {}\n", quote(&connection.token)?));
            } else {
                output.push(format!(
                    "username = {}\ntoken = {}\n",
                    quote(&connection.username)?,
                    quote(&connection.token)?
                ));
            }
            output.push(format!(
                "\n[{prefix}connections.{name}]\nendpoint = {}\n",
                quote(&connection.endpoint)?
            ));
            if let Some(id) = connection.project_id {
                output.push(format!("project_id = {id}\n"));
            }
        }
    }
    Ok(output.concat())
}
/// JSON basic-string escaping is also valid for TOML basic strings.
fn quote(value: &str) -> Result<String, String> {
    serde_json::to_string(value).map_err(|_| "Cannot encode legacy connection field".into())
}
/// Write the explicitly requested new file privately, leaving the input intact.
pub(super) fn run(args: &MigrateSecretsArgs) -> Result<(), String> {
    if args.output.extension().is_none_or(|ext| ext != "toml") {
        return Err("The output must be a .toml file".into());
    }
    let source =
        std::fs::read(&args.input).map_err(|e| format!("Cannot read legacy file: {}", e.kind()))?;
    let converted = convert(&source)?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut output = options
        .open(&args.output)
        .map_err(|e| format!("Cannot create TOML file: {}", e.kind()))?;
    output
        .write_all(converted.as_bytes())
        .map_err(|e| format!("Cannot write TOML file: {}", e.kind()))?;
    output
        .sync_all()
        .map_err(|e| format!("Cannot sync TOML file: {}", e.kind()))?;
    println!(
        "Created {}. Set SCRYR_SECRETS_FILE and remove SCRYR_METRICS_CONNECTIONS_FILE. Match authentication variable names to the TOML tables, then run scryr check.",
        args.output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_preserves_credentials_and_never_overwrites()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let input = folder.path().join("old.json");
        let output = folder.path().join("scryr.secrets.toml");
        std::fs::write(
            &input,
            r#"{"org":{"grafana":{"endpoint":"https://metrics.example","username":"user","token":"test\\token"},"posthog":{"endpoint":"https://us.posthog.com","projectId":123,"token":"test-key"}}}"#,
        )?;
        let args = MigrateSecretsArgs { input, output };
        run(&args)?;
        let secrets = crystal_core::integration_secrets::SecretsFile::read(&args.output)?;
        assert!(secrets.scope("org").authentication["grafana"].supports("grafana"));
        assert!(secrets.scope("org").authentication["posthog"].supports("posthog"));
        let previous = std::fs::read(&args.output)?;
        assert!(run(&args).is_err());
        assert_eq!(std::fs::read(&args.output)?, previous);
        assert!(args.input.exists());
        Ok(())
    }
}
