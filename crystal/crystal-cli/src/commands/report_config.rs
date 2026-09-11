//! Resolve reporting configuration from manifest declarations.
use crate::args::ReportSourceArgs;
use serde_json::Value;

/// Match a public manifest selector without guessing between multiple matches.
pub(crate) fn matches(value: &Value, selector: &str) -> bool {
    ["manifestId", "name", "variable_name"]
        .iter()
        .any(|key| value[key].as_str() == Some(selector))
}
/// Load the selected manifest only when explicit-ID compatibility mode is not used.
pub(crate) fn resolve(
    source: &ReportSourceArgs,
    explicit_id: &str,
) -> Result<Option<(Value, std::path::PathBuf)>, String> {
    if !explicit_id.is_empty() && source.path.is_none() && source.manifest.is_none() {
        return Ok(None);
    }
    let common = crate::args::GenerateCommonArgs {
        manifest_file: source.path.clone().unwrap_or_else(|| "index.scry".into()),
        manifest_dir: source.manifest_dir.clone(),
        scryr_dir: source.scryr_dir.clone(),
        graphql_url: None,
        clerk_org_id: None,
        git_commit_sha: None,
    };
    let project = super::workflow::Project::new(common)?;
    let envelope: Value = serde_json::from_str(&project.json()?).map_err(|e| e.to_string())?;
    let values = envelope["manifests"]
        .as_array()
        .ok_or("Missing manifest declarations")?;
    let selector = source
        .manifest
        .as_deref()
        .or_else(|| (!explicit_id.is_empty()).then_some(explicit_id));
    let selected: Vec<_> = values
        .iter()
        .filter(|v| selector.is_none_or(|s| matches(v, s)))
        .collect();
    match selected.as_slice() {
        [manifest] => {
            if manifest["manifestId"].as_str().is_none_or(str::is_empty) {
                return Err("Reporting requires a stable manifest_id in index.scry".into());
            }
            Ok(Some((
                (*manifest).clone(),
                project
                    .file
                    .parent()
                    .ok_or("Missing source directory")?
                    .to_path_buf(),
            )))
        }
        [] => Err("No manifest matches the reporting selector".into()),
        _ => Err("Multiple manifests found; select one with --manifest".into()),
    }
}
