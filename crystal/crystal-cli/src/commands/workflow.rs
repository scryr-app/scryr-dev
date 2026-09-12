//! Shared manifest preparation, validation, and publication.
use super::generate::executor::{ManifestExecutor, prepare_manifest_executor};
use super::generate::upload::{
    GeneratedArtifactPersistence, ManifestLocation, default_local_graphql_url,
    persist_generated_artifacts, upload_bearer_token,
};
use crate::args::{GenerateCommonArgs, GenerateOutput, GenerateRequest, resolve_generate_request};
use crate::manifest_paths::{derive_manifest_artifact_key, resolve_manifest_file};
use crate::manifest_python::ManifestPythonMode;
use crystal_core::generation::{GeneratedMapArtifact, map_artifacts_from_manifest_json};
use std::path::PathBuf;

/// A prepared environment and resolved source location.
pub(crate) struct Project {
    /// Command options.
    pub args: GenerateRequest,
    /// Project directory.
    pub root: PathBuf,
    /// Manifest entrypoint.
    pub file: PathBuf,
    /// Credentials remain at the original entrypoint when editing staged source.
    pub secrets_file: PathBuf,
    /// Managed Python adapter.
    pub executor: ManifestExecutor,
}
impl Project {
    /// Resolve the manifest and provision its managed runtime.
    pub(crate) fn new(common: GenerateCommonArgs) -> Result<Self, String> {
        Self::from_request(resolve_generate_request(
            GenerateOutput::Upload,
            common,
            None,
        ))
    }
    /// Prepare an existing generation request.
    pub(crate) fn from_request(args: GenerateRequest) -> Result<Self, String> {
        let root = args
            .manifest_dir
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let file = resolve_manifest_file(&root, &args.manifest_file)?;
        let executor = prepare_manifest_executor(&args, &root)?;
        let secrets_file = crystal_core::integration_secrets::secrets_path(&file);
        Ok(Self {
            args,
            root,
            file,
            secrets_file,
            executor,
        })
    }
    /// Run source tooling and retain its diagnostics on stderr.
    pub(crate) fn tool(&self, mode: &str) -> Result<(), String> {
        eprint!(
            "{}",
            self.executor
                .tool(&self.root, &self.file, mode)
                .map_err(|e| {
                    let action = if mode == "--format-check" {
                        "\nRun `scryr format` to apply these changes."
                    } else {
                        ""
                    };
                    format!("{} failed:\n{e}{action}", mode.trim_start_matches('-'))
                })?
        );
        Ok(())
    }
    /// Execute the manifest once and return its serialized envelope.
    pub(crate) fn json(&self) -> Result<String, String> {
        self.executor
            .run(&self.root, &self.file, ManifestPythonMode::Json)
            .map_err(|e| format!("Manifest execution/JSON serialization failed:\n{e}"))
    }
    /// Check source and runtime constraints and retain the resulting artifact.
    pub(crate) fn check(&self) -> Result<Checked, String> {
        for mode in ["--format-check", "--lint", "--typecheck"] {
            self.tool(mode)?;
        }
        let json = self.json()?;
        let key = derive_manifest_artifact_key(&self.root, &self.file);
        let maps = validate_json(&json, &key)?;
        let envelope = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let secrets =
            if self.secrets_file.exists() || std::env::var_os("SCRYR_SECRETS_FILE").is_some() {
                crystal_core::integration_secrets::SecretsFile::read(&self.secrets_file)?
            } else {
                crystal_core::integration_secrets::SecretsFile::default()
            };
        secrets.validate(
            &envelope,
            self.args.clerk_org_id.as_deref().unwrap_or("local-dev-org"),
        )?;
        let schema = self
            .executor
            .run(&self.root, &self.file, ManifestPythonMode::Schema)?;
        eprintln!(
            "Checked {}: formatting, lint, types, execution, Scryr rules, and TOML credentials passed",
            self.file.display()
        );
        Ok(Checked { json, maps, schema })
    }
}
/// Validated artifacts, reused without re-executing user code.
pub(crate) struct Checked {
    /// Whole generated envelope.
    pub json: String,
    /// Diagram-scoped uploads.
    pub maps: Vec<GeneratedMapArtifact>,
    /// SDK schema.
    pub schema: String,
}
/// Validate new diagram artifacts while retaining legacy persistence readers.
pub(crate) fn validate_json(json: &str, key: &str) -> Result<Vec<GeneratedMapArtifact>, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;
    if value["diagrams"].as_array().is_none_or(Vec::is_empty) {
        return Err("Scryr rules: define at least one public Diagram in index.scry".into());
    }
    map_artifacts_from_manifest_json(json, key).map_err(|e| format!("Scryr rules: {e}"))
}
/// Publish previously checked data to the selected deployment.
pub(crate) async fn publish(project: &Project, checked: Checked) -> Result<String, String> {
    let target = project
        .args
        .graphql_url
        .clone()
        .unwrap_or_else(default_local_graphql_url);
    let mut url = super::report::endpoint(&target)?;
    let token = upload_bearer_token(&target).await?;
    persist_generated_artifacts(GeneratedArtifactPersistence {
        graphql_url: &target,
        bearer_token: token.as_deref(),
        clerk_org_id: project.args.clerk_org_id.as_deref(),
        pydantic_schema: checked.schema,
        map_artifacts: checked.maps,
        manifest_location: ManifestLocation::from_paths(&project.root, &project.file),
        git_commit_sha: project.args.git_commit_sha.clone(),
    })
    .await?;
    url.set_path("/");
    url.set_query(None);
    url.set_fragment(None);
    eprintln!("Diagrams available at {url}");
    Ok(url.into())
}
/// Check and publish a manifest.
pub(crate) async fn push(common: GenerateCommonArgs) -> Result<(), String> {
    let project = Project::new(common)?;
    let checked = project.check()?;
    publish(&project, checked).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_json;
    use serde_json::json;

    #[test]
    fn missing_or_empty_diagrams_are_rejected() {
        for value in [
            json!({"manifests": []}),
            json!({"diagrams": [], "manifests": []}),
        ] {
            assert!(validate_json(&value.to_string(), "index").is_err());
        }
    }

    #[test]
    fn diagrams_must_resolve_public_manifests_and_have_unique_ids() {
        let mut value = json!({"manifests":[{"name":"API"}], "diagrams":[{
            "name":"System", "variable_name":"system", "manifests":[{"name":"API"}]
        }]});
        assert!(validate_json(&value.to_string(), "index").is_ok());
        value["diagrams"][0]["manifests"][0]["name"] = json!("Unknown");
        assert!(validate_json(&value.to_string(), "index").is_err());
        value["diagrams"][0]["manifests"][0]["name"] = json!("API");
        value["diagrams"] = json!([value["diagrams"][0], value["diagrams"][0]]);
        assert!(validate_json(&value.to_string(), "index").is_err());
    }
}
