//! Shared generated manifest write row construction.

use super::metadata::ManifestUploadMetadata;
use crate::manifest::{ManifestRequestContext, UpsertGeneratedManifestInput};
use uuid::Uuid;

/// Values used to update the current row and append one upload ledger row.
pub(super) struct ManifestWriteRows {
    /// Current generated manifest row values.
    pub(super) current: GeneratedManifestRow,
    /// Upload ledger row values.
    pub(super) upload: GeneratedManifestUploadRow,
}

impl ManifestWriteRows {
    /// Build owned write rows shared by `SQLite` and Turso/libSQL bind layers.
    pub(super) fn build(
        input: &UpsertGeneratedManifestInput,
        metadata: &ManifestUploadMetadata,
        request_context: &ManifestRequestContext,
    ) -> Self {
        let common = GeneratedManifestCommonFields {
            artifact_kind: input.artifact_kind.as_str().to_string(),
            artifact_key: input.artifact_key.clone(),
            clerk_org_id: request_context.clerk_org_id.clone(),
            org_slug: request_context.clerk_org_slug.clone(),
            uploaded_by_clerk_user_id: request_context.clerk_user_id.clone(),
            folder_path: metadata.folder_path.clone(),
            file_name: metadata.file_name.clone(),
            scry_identifier: metadata.scry_identifier.clone(),
            name: metadata.name.clone(),
            git_commit_sha: metadata.git_commit_sha.clone(),
            content: input.content.clone(),
        };
        let current = GeneratedManifestRow {
            id: Uuid::new_v4().to_string(),
            common: common.clone(),
        };
        let upload = GeneratedManifestUploadRow {
            id: Uuid::new_v4().to_string(),
            common,
        };

        Self { current, upload }
    }
}

/// Current generated manifest row values.
pub(super) struct GeneratedManifestRow {
    /// Candidate row id used when inserting.
    pub(super) id: String,
    /// Shared manifest fields.
    pub(super) common: GeneratedManifestCommonFields,
}

/// Upload ledger row values.
pub(super) struct GeneratedManifestUploadRow {
    /// Upload ledger id.
    pub(super) id: String,
    /// Shared manifest fields.
    pub(super) common: GeneratedManifestCommonFields,
}

/// Manifest fields bound by both the current and upload rows.
#[derive(Clone)]
pub(super) struct GeneratedManifestCommonFields {
    /// Logical artifact kind.
    pub(super) artifact_kind: String,
    /// Persisted manifest artifact key.
    pub(super) artifact_key: String,
    /// Owning organization id.
    pub(super) clerk_org_id: String,
    /// Owning organization slug.
    pub(super) org_slug: Option<String>,
    /// Uploading user id.
    pub(super) uploaded_by_clerk_user_id: String,
    /// Manifest folder path.
    pub(super) folder_path: String,
    /// Manifest file name.
    pub(super) file_name: String,
    /// Stable Scryr map identifier.
    pub(super) scry_identifier: String,
    /// Human-readable map name.
    pub(super) name: String,
    /// Git commit SHA.
    pub(super) git_commit_sha: Option<String>,
    /// Generated artifact content.
    pub(super) content: String,
}
