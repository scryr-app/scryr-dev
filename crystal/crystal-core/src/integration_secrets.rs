//! TOML credentials shared by checking, local serving, and hosted runtime queries.
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Grafana basic authentication; never serialized into a manifest.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrafanaSecret {
    /// Grafana user or stack ID.
    pub username: String,
    /// Grafana access token.
    pub token: String,
}
/// `PostHog` personal API authentication.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostHogSecret {
    /// `PostHog` personal API key.
    pub api_key: String,
}
/// GitHub API authentication.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubSecret {
    /// GitHub access token.
    pub token: String,
}
/// Exact provider credential schemas, selected by their required fields.
#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum Authentication {
    /// Grafana username and token.
    Grafana(GrafanaSecret),
    /// `PostHog` API key.
    PostHog(PostHogSecret),
    /// GitHub token.
    GitHub(GitHubSecret),
}
impl Authentication {
    /// Whether all required fields are populated for the requested provider.
    #[must_use]
    pub fn supports(&self, kind: &str) -> bool {
        match (self, kind) {
            (Self::Grafana(s), "grafana" | "prometheus") => {
                !s.username.trim().is_empty() && !s.token.trim().is_empty()
            }
            (Self::PostHog(s), "posthog") => !s.api_key.trim().is_empty(),
            (Self::GitHub(s), "github" | "github_actions") => !s.token.trim().is_empty(),
            _ => false,
        }
    }
}
/// Server operator approval for a hosted connection's destination.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionApproval {
    /// Approved endpoint, which must match index.scry.
    pub endpoint: String,
    /// Approved `PostHog` project, which must match index.scry.
    pub project_id: Option<u64>,
}
/// Organization-specific credentials and optional hosted destination approvals.
#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretScope {
    /// Named credential tables, matching authentication declaration variables.
    #[serde(default)]
    pub authentication: BTreeMap<String, Authentication>,
    /// Hosted destination restrictions, keyed by authentication name.
    #[serde(default)]
    pub connections: BTreeMap<String, ConnectionApproval>,
}
/// A secrets file contains local credentials or explicitly organization-scoped credentials.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretsFile {
    /// Credentials used exclusively with local development authentication.
    #[serde(default)]
    pub authentication: BTreeMap<String, Authentication>,
    /// Destination approvals for a local server running without an index.scry workspace.
    #[serde(default)]
    pub connections: BTreeMap<String, ConnectionApproval>,
    /// Organization-isolated hosted credentials and destination approvals.
    #[serde(default)]
    pub organizations: BTreeMap<String, SecretScope>,
}
impl SecretsFile {
    /// Read TOML without including secret source excerpts in diagnostics.
    ///
    /// # Errors
    /// Returns a redacted diagnostic for inaccessible files or invalid TOML/schema.
    pub fn read(path: &Path) -> Result<Self, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e.kind()))?;
        let file: Self = toml_edit::de::from_str(&source)
            .map_err(|_| format!("Invalid TOML or credential schema in {}", path.display()))?;
        for (name, auth) in file
            .authentication
            .iter()
            .chain(file.organizations.values().flat_map(|s| &s.authentication))
        {
            if name.is_empty()
                || !["grafana", "posthog", "github"]
                    .iter()
                    .any(|k| auth.supports(k))
            {
                return Err(format!("Empty credential fields in authentication.{name}"));
            }
        }
        Ok(file)
    }
    /// Resolve only the authenticated organization's credentials.
    #[must_use]
    pub fn scope(&self, org: &str) -> SecretScope {
        if org == "local-dev-org" {
            SecretScope {
                authentication: self.authentication.clone(),
                connections: self.connections.clone(),
            }
        } else {
            self.organizations.get(org).cloned().unwrap_or_default()
        }
    }
    /// Check references in generated manifests without making network requests.
    ///
    /// # Errors
    /// Returns the missing or mismatched declaration name, never a credential value.
    pub fn validate(&self, envelope: &Value, org: &str) -> Result<(), String> {
        let scope = self.scope(org);
        if let Some(manifests) = envelope["manifests"].as_array() {
            for manifest in manifests {
                if let Some(integrations) = manifest["integrations"].as_array() {
                    for integration in integrations {
                        let auth = &integration["authentication"];
                        if auth.is_null() {
                            continue;
                        }
                        let name = auth["id"]
                            .as_str()
                            .ok_or("Authentication requires a variable name")?;
                        let kind = auth["kind"]
                            .as_str()
                            .ok_or("Authentication requires a provider")?;
                        if !scope
                            .authentication
                            .get(name)
                            .is_some_and(|s| s.supports(kind))
                        {
                            return Err(format!(
                                "Missing or incompatible authentication.{name} for {kind} in scryr.secrets.toml"
                            ));
                        }
                    }
                }
                for source in [&manifest["metrics"]["provider"], &manifest["analytics"]] {
                    if let Some(name) = source["credentials"]["name"].as_str() {
                        let kind = source["kind"].as_str().unwrap_or_default();
                        if !scope
                            .authentication
                            .get(name)
                            .is_some_and(|s| s.supports(kind))
                        {
                            return Err(format!(
                                "Missing or incompatible authentication.{name} in scryr.secrets.toml"
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
/// Choose a secrets file relative to the entrypoint, or an explicit environment override.
#[must_use]
pub fn secrets_path(entrypoint: &Path) -> PathBuf {
    std::env::var_os("SCRYR_SECRETS_FILE").map_or_else(
        || entrypoint.with_file_name("scryr.secrets.toml"),
        PathBuf::from,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_provider_schemas_and_organization_isolation() -> Result<(), Box<dyn std::error::Error>>
    {
        let folder = tempfile::tempdir()?;
        let path = folder.path().join("scryr.secrets.toml");
        std::fs::write(
            &path,
            r#"
[authentication.grafana_authentication]
username = "test-user"
token = "test-token"
[authentication.posthog_authentication]
api_key = "test-key"
[organizations.org_a.authentication.github_authentication]
token = "test-github-token"
[organizations.org_a.connections.github_authentication]
endpoint = "https://api.github.com"
"#,
        )?;
        let secrets = SecretsFile::read(&path)?;
        let envelope = serde_json::json!({"manifests":[{"integrations":[
            {"authentication":{"id":"grafana_authentication","kind":"grafana"}},
            {"authentication":{"id":"posthog_authentication","kind":"posthog"}}
        ]}]});
        secrets.validate(&envelope, "local-dev-org")?;
        assert!(secrets.validate(&envelope, "org_a").is_err());
        assert!(secrets.scope("org_b").authentication.is_empty());
        assert!(secrets.scope("org_a").authentication["github_authentication"].supports("github"));
        assert!(
            !secrets.scope("local-dev-org").authentication["posthog_authentication"]
                .supports("grafana")
        );
        Ok(())
    }

    #[test]
    fn invalid_secrets_never_appear_in_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let path = folder.path().join("scryr.secrets.toml");
        for source in [
            "[authentication.bad]\ntoken = \"sensitive-value\n",
            "[authentication.bad]\ntoken = \"sensitive-value\"\nunknown = true",
            "{\"token\":\"sensitive-value\"}",
            "[authentication.bad]\ntoken = \"\"",
        ] {
            std::fs::write(&path, source)?;
            let error = SecretsFile::read(&path)
                .err()
                .ok_or("Expected invalid secrets")?;
            assert!(!error.contains("sensitive-value"));
        }
        SecretsFile::default().validate(
            &serde_json::json!({"manifests":[{"cards":[]}]}),
            "local-dev-org",
        )?;
        Ok(())
    }
}
