//! GraphQL map listing types.

use async_graphql::SimpleObject;

/// A persisted Scryr map artifact available in storage.
#[derive(SimpleObject, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[graphql(name = "ScryrMap")]
pub struct ScryrMap {
    /// Public map identifier used by the UI.
    pub id: String,
    /// Artifact key used by legacy sample-based lookups.
    pub key: String,
    /// Stable Scryr identifier, normally the top-level Diagram variable name.
    pub scry_identifier: String,
    /// Human-readable map name from the declared Diagram.
    pub name: String,
    /// Clerk organization id that owns the uploaded manifest.
    pub clerk_org_id: String,
    /// Clerk organization slug cached from the uploader's active session.
    pub org_slug: Option<String>,
    /// Folder path containing the uploaded manifest file.
    pub folder_path: String,
    /// Uploaded manifest file name.
    pub file_name: String,
    /// Git commit SHA associated with the upload, when supplied.
    pub git_commit_sha: Option<String>,
    /// Last update timestamp rendered by the database.
    pub updated_at: String,
}
