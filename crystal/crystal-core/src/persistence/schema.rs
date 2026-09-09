//! Schema bootstrap and migrations for generated manifest persistence.

use super::models::DatabasePool;
use sqlx::SqlitePool;

const CREATE_GENERATED_MANIFESTS_SQL: &str = r"
    CREATE TABLE IF NOT EXISTS generated_manifests (
        id TEXT PRIMARY KEY,
        artifact_kind TEXT NOT NULL,
        manifeset_file_name TEXT NOT NULL DEFAULT '',
        organization TEXT NOT NULL DEFAULT '',
        clerk_org_id TEXT NOT NULL DEFAULT '',
        org_slug TEXT,
        uploaded_by_clerk_user_id TEXT NOT NULL DEFAULT '',
        folder_path TEXT NOT NULL DEFAULT '',
        file_name TEXT NOT NULL DEFAULT '',
        scry_identifier TEXT NOT NULL DEFAULT '',
        name TEXT NOT NULL DEFAULT '',
        git_commit_sha TEXT,
        content TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
";

const CREATE_LEGACY_KEY_INDEX_SQL: &str = r"
    CREATE UNIQUE INDEX IF NOT EXISTS generated_manifests_legacy_key_idx
    ON generated_manifests (artifact_kind, clerk_org_id, manifeset_file_name)
    WHERE scry_identifier = ''
";

const CREATE_SCRY_IDENTIFIER_INDEX_SQL: &str = r"
    CREATE UNIQUE INDEX IF NOT EXISTS generated_manifests_scry_identifier_idx
    ON generated_manifests (artifact_kind, clerk_org_id, scry_identifier)
    WHERE scry_identifier <> ''
";

const CREATE_UPLOAD_LEDGER_SQL: &str = r"
    CREATE TABLE IF NOT EXISTS generated_manifest_uploads (
        id TEXT PRIMARY KEY,
        generated_manifest_id TEXT NOT NULL
            REFERENCES generated_manifests(id)
            ON DELETE CASCADE,
        artifact_kind TEXT NOT NULL,
        manifeset_file_name TEXT NOT NULL DEFAULT '',
        organization TEXT NOT NULL DEFAULT '',
        clerk_org_id TEXT NOT NULL DEFAULT '',
        org_slug TEXT,
        uploaded_by_clerk_user_id TEXT NOT NULL DEFAULT '',
        folder_path TEXT NOT NULL DEFAULT '',
        file_name TEXT NOT NULL DEFAULT '',
        scry_identifier TEXT NOT NULL DEFAULT '',
        name TEXT NOT NULL DEFAULT '',
        git_commit_sha TEXT,
        content TEXT NOT NULL,
        uploaded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
";

const CREATE_UPLOAD_LEDGER_INDEX_SQL: &str = r"
    CREATE INDEX IF NOT EXISTS generated_manifest_uploads_current_idx
    ON generated_manifest_uploads (generated_manifest_id, uploaded_at DESC)
";

const SCHEMA_STATEMENTS: &[(&str, &str)] = &[
    (
        "create manifest action history",
        super::action_history::CREATE_TABLE,
    ),
    (
        "create generated_manifests table",
        CREATE_GENERATED_MANIFESTS_SQL,
    ),
    (
        "index legacy generated manifest keys",
        CREATE_LEGACY_KEY_INDEX_SQL,
    ),
    (
        "index generated manifest identifiers",
        CREATE_SCRY_IDENTIFIER_INDEX_SQL,
    ),
    (
        "create generated_manifest_uploads table",
        CREATE_UPLOAD_LEDGER_SQL,
    ),
    (
        "index generated manifest uploads",
        CREATE_UPLOAD_LEDGER_INDEX_SQL,
    ),
];

/// Create the local `SQLite` artifact storage tables when they do not already exist.
async fn ensure_sqlite_table(pool: &SqlitePool) -> Result<(), String> {
    for (description, statement) in SCHEMA_STATEMENTS {
        sqlx::query(*statement)
            .execute(pool)
            .await
            .map_err(|error| format!("Failed to {description} in local SQLite storage: {error}"))?;
    }

    Ok(())
}

/// Create the Turso/libSQL artifact storage tables when they do not already exist.
async fn ensure_turso_table(database: &libsql::Database) -> Result<(), String> {
    let connection = database
        .connect()
        .map_err(|error| format!("Failed to open Turso schema connection: {error}"))?;

    for (description, statement) in SCHEMA_STATEMENTS {
        connection
            .execute(statement, ())
            .await
            .map_err(|error| format!("Failed to {description} in Turso storage: {error}"))?;
    }

    Ok(())
}

/// Create the artifact storage table when it does not already exist.
///
/// # Errors
///
/// Returns an error if any schema bootstrap or migration step fails.
pub(crate) async fn ensure_table(pool: &DatabasePool) -> Result<(), String> {
    match pool {
        DatabasePool::Turso(database) => ensure_turso_table(database).await,
        DatabasePool::Sqlite(pool) => ensure_sqlite_table(pool).await,
    }
}
