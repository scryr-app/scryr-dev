//! Shared SQL statements for generated manifest writes.

/// SQLite-compatible upsert used when the upload has no stable Scryr identifier.
pub(super) const UPSERT_BY_ARTIFACT_KEY_SQL: &str = r"
    INSERT INTO generated_manifests (
        id, artifact_kind, manifeset_file_name, clerk_org_id, org_slug,
        uploaded_by_clerk_user_id, folder_path, file_name, scry_identifier,
        name, git_commit_sha, content
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT (artifact_kind, clerk_org_id, manifeset_file_name)
    WHERE scry_identifier = ''
    DO UPDATE SET
        manifeset_file_name = excluded.manifeset_file_name,
        clerk_org_id = excluded.clerk_org_id,
        org_slug = excluded.org_slug,
        uploaded_by_clerk_user_id = excluded.uploaded_by_clerk_user_id,
        folder_path = excluded.folder_path,
        file_name = excluded.file_name,
        scry_identifier = excluded.scry_identifier,
        name = excluded.name,
        git_commit_sha = excluded.git_commit_sha,
        content = excluded.content,
        updated_at = CURRENT_TIMESTAMP
    RETURNING id
";

/// SQLite-compatible upsert used when the upload has a stable Scryr identifier.
pub(super) const UPSERT_BY_SCRY_IDENTIFIER_SQL: &str = r"
    INSERT INTO generated_manifests (
        id, artifact_kind, manifeset_file_name, clerk_org_id, org_slug,
        uploaded_by_clerk_user_id, folder_path, file_name, scry_identifier,
        name, git_commit_sha, content
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT (artifact_kind, clerk_org_id, scry_identifier)
    WHERE scry_identifier <> ''
    DO UPDATE SET
        manifeset_file_name = excluded.manifeset_file_name,
        clerk_org_id = excluded.clerk_org_id,
        org_slug = excluded.org_slug,
        uploaded_by_clerk_user_id = excluded.uploaded_by_clerk_user_id,
        folder_path = excluded.folder_path,
        file_name = excluded.file_name,
        scry_identifier = excluded.scry_identifier,
        name = excluded.name,
        git_commit_sha = excluded.git_commit_sha,
        content = excluded.content,
        updated_at = CURRENT_TIMESTAMP
    RETURNING id
";

/// SQLite-compatible insert used for the append-only upload ledger.
pub(super) const INSERT_UPLOAD_LEDGER_SQL: &str = r"
    INSERT INTO generated_manifest_uploads (
        id, generated_manifest_id, artifact_kind, manifeset_file_name, clerk_org_id,
        org_slug, uploaded_by_clerk_user_id, folder_path, file_name,
        scry_identifier, name, git_commit_sha, content
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
";
