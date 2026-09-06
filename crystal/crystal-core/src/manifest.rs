//! Shared manifest input and request-context models for generated manifest persistence.

use async_graphql::{Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Logical kind for a generated artifact persisted to storage.
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[graphql(rename_items = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    /// Generated instance/value output.
    Value,
    /// Generated schema or type-definition output.
    Schema,
}

impl ArtifactKind {
    /// Database representation for the artifact kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Schema => "schema",
        }
    }
}

/// Input payload for upserting a generated manifest artifact.
#[derive(InputObject, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpsertGeneratedManifestInput {
    /// Logical artifact kind.
    pub artifact_kind: ArtifactKind,
    /// Persisted manifest artifact key.
    pub artifact_key: String,
    /// Folder path containing the uploaded manifest file.
    pub folder_path: Option<String>,
    /// Uploaded manifest file name.
    pub file_name: Option<String>,
    /// Stable Scryr identifier, normally the top-level Diagram variable name.
    pub scry_identifier: Option<String>,
    /// Human-readable map name from the declared Diagram.
    pub name: Option<String>,
    /// Git commit SHA associated with the upload, when supplied by the caller.
    pub git_commit_sha: Option<String>,
    /// Full generated artifact content.
    pub content: String,
}

/// Payload returned after upserting a generated manifest artifact.
#[derive(SimpleObject, Serialize, Deserialize, Debug, Clone)]
pub struct UpsertGeneratedManifestPayload {
    /// Persisted row identifier.
    pub id: Uuid,
}

/// Authenticated request data used to scope generated manifest reads and writes.
///
/// The field names match existing `clerk_*` storage columns for compatibility,
/// but callers should treat this as provider-normalized user and organization
/// context.
#[derive(Clone, Debug)]
pub struct ManifestRequestContext {
    /// Provider user id for the authenticated user.
    pub clerk_user_id: String,
    /// Provider organization id for the active organization.
    pub clerk_org_id: String,
    /// Organization slug for the active organization, when present.
    pub clerk_org_slug: Option<String>,
    /// Organization role for the active organization, when present.
    pub clerk_org_role: Option<String>,
    /// Provider organization permissions for the active organization.
    pub clerk_org_permissions: Vec<String>,
}

impl ManifestRequestContext {
    /// Return whether this principal can write generated manifests.
    #[must_use]
    pub fn can_write_generated_manifests(&self) -> bool {
        self.clerk_org_role
            .as_deref()
            .is_some_and(|role| matches!(role, "admin" | "org:admin"))
            || self.clerk_org_permissions.iter().any(|permission| {
                matches!(
                    permission.as_str(),
                    "maps:write" | "scryr:maps:write" | "org:admin:maps:write"
                )
            })
    }
}
