//! Upload metadata normalization for generated manifest persistence.

use crate::manifest::UpsertGeneratedManifestInput;

/// Normalized location metadata for a manifest upload.
pub(super) struct ManifestUploadMetadata {
    /// Folder path containing the file.
    pub(super) folder_path: String,
    /// Uploaded file name.
    pub(super) file_name: String,
    /// Stable Scryr identifier, normally the top-level Diagram variable name.
    pub(super) scry_identifier: String,
    /// Human-readable map name from the declared Diagram.
    pub(super) name: String,
    /// Optional git commit SHA for the upload.
    pub(super) git_commit_sha: Option<String>,
}

impl ManifestUploadMetadata {
    /// Build metadata from explicit GraphQL input with legacy artifact-key fallback.
    pub(super) fn from_input(input: &UpsertGeneratedManifestInput) -> Self {
        let (fallback_folder, fallback_file) = split_artifact_key(&input.artifact_key);
        Self {
            folder_path: clean_optional_text(input.folder_path.as_deref())
                .unwrap_or(fallback_folder),
            file_name: clean_optional_text(input.file_name.as_deref()).unwrap_or(fallback_file),
            scry_identifier: clean_optional_text(input.scry_identifier.as_deref())
                .unwrap_or_default(),
            name: clean_optional_text(input.name.as_deref()).unwrap_or_default(),
            git_commit_sha: clean_optional_text(input.git_commit_sha.as_deref()),
        }
    }
}

/// Split a legacy artifact key into folder path and file name fallback values.
fn split_artifact_key(artifact_key: &str) -> (String, String) {
    let trimmed = artifact_key.trim_matches('/');
    let Some((folder_path, file_name)) = trimmed.rsplit_once('/') else {
        return (String::new(), trimmed.to_string());
    };

    (folder_path.to_string(), file_name.to_string())
}

/// Normalize optional input text, treating blank strings as absent.
fn clean_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::ManifestUploadMetadata;
    use crate::manifest::UpsertGeneratedManifestInput;
    use crate::persistence::ArtifactKind;

    #[test]
    fn upload_metadata_prefers_explicit_location_fields() {
        let input = UpsertGeneratedManifestInput {
            artifact_kind: ArtifactKind::Value,
            artifact_key: "legacy/key".to_string(),
            folder_path: Some("services/catalog".to_string()),
            file_name: Some("index.scry".to_string()),
            scry_identifier: Some("catalog_diagram".to_string()),
            name: Some("Catalog".to_string()),
            git_commit_sha: Some("abc123".to_string()),
            content: "{}".to_string(),
        };

        let metadata = ManifestUploadMetadata::from_input(&input);

        assert_eq!(metadata.folder_path, "services/catalog");
        assert_eq!(metadata.file_name, "index.scry");
        assert_eq!(metadata.scry_identifier, "catalog_diagram");
        assert_eq!(metadata.name, "Catalog");
        assert_eq!(metadata.git_commit_sha.as_deref(), Some("abc123"));
    }

    #[test]
    fn upload_metadata_falls_back_to_legacy_artifact_key() {
        let input = UpsertGeneratedManifestInput {
            artifact_kind: ArtifactKind::Value,
            artifact_key: "samples/open_saas".to_string(),
            folder_path: None,
            file_name: None,
            scry_identifier: None,
            name: None,
            git_commit_sha: None,
            content: "{}".to_string(),
        };

        let metadata = ManifestUploadMetadata::from_input(&input);

        assert_eq!(metadata.folder_path, "samples");
        assert_eq!(metadata.file_name, "open_saas");
        assert_eq!(metadata.scry_identifier, "");
        assert_eq!(metadata.name, "");
        assert_eq!(metadata.git_commit_sha, None);
    }
}
