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
        "create organization sample seed markers",
        "CREATE TABLE IF NOT EXISTS organization_sample_seeds (clerk_org_id TEXT PRIMARY KEY, seeded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)",
    ),
    (
        "create evidence history",
        super::evidence::CREATE_OBSERVATIONS,
    ),
    ("create collector status", super::evidence::CREATE_STATUSES),
    ("index evidence", super::evidence::CREATE_INDEX),
    (
        "index workflow evidence sources",
        super::evidence::WORKFLOW_SOURCES_INDEX,
    ),
    (
        "create workflow projection",
        super::evidence::workflows::CREATE,
    ),
    (
        "index workflow projection",
        super::evidence::workflows::INDEX,
    ),
    (
        "index workflow projection ownership",
        super::evidence::workflows::OWNERSHIP_INDEX,
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

/// Breaking storage generation. Existing databases are never upgraded or erased implicitly.
const SCHEMA_VERSION: &str = "3";
const TABLES: &str =
    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";
const VERSION: &str = "SELECT version FROM scryr_schema WHERE singleton=1";
const CREATE_VERSION: &str = "CREATE TABLE scryr_schema (singleton INTEGER PRIMARY KEY CHECK(singleton=1), version TEXT NOT NULL)";
const INSERT_VERSION: &str = "INSERT INTO scryr_schema (singleton,version) VALUES (1,'3')";
fn incompatible() -> String {
    format!(
        "Incompatible Scryr database schema; expected {SCHEMA_VERSION}. Keep a backup and configure a fresh empty database with SCRYR_SQLITE_PATH or a new Turso database, then reimport index.scry. Existing data was not changed."
    )
}
fn validate_version(versions: &[String]) -> Result<(), String> {
    if versions == [SCHEMA_VERSION] {
        Ok(())
    } else {
        Err(incompatible())
    }
}
async fn ensure_sqlite_table(pool: &SqlitePool) -> Result<(), String> {
    let tables: Vec<String> = sqlx::query_scalar(TABLES)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    if tables.iter().any(|name| name == "scryr_schema") {
        return validate_version(
            &sqlx::query_scalar::<_, String>(VERSION)
                .fetch_all(pool)
                .await
                .map_err(|e| e.to_string())?,
        );
    }
    if !tables.is_empty() {
        return Err(incompatible());
    }
    // Serialize competing fresh initializers and publish the version atomically with all tables.
    let mut tx = pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(|e| e.to_string())?;
    let tables: Vec<String> = sqlx::query_scalar(TABLES)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    if tables.iter().any(|name| name == "scryr_schema") {
        let versions = sqlx::query_scalar::<_, String>(VERSION)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        validate_version(&versions)?;
    } else {
        if !tables.is_empty() {
            return Err(incompatible());
        }
        for statement in [CREATE_VERSION, INSERT_VERSION]
            .into_iter()
            .chain(SCHEMA_STATEMENTS.iter().map(|(_, sql)| *sql))
        {
            sqlx::query(statement)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().await.map_err(|e| e.to_string())
}
async fn text_rows(mut rows: libsql::Rows) -> Result<Vec<String>, String> {
    let mut out = vec![];
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        out.push(row.get(0).map_err(|e| e.to_string())?);
    }
    Ok(out)
}
async fn ensure_turso_table(database: &libsql::Database) -> Result<(), String> {
    let c = database.connect().map_err(|e| e.to_string())?;
    let tables = text_rows(c.query(TABLES, ()).await.map_err(|e| e.to_string())?).await?;
    if tables.iter().any(|name| name == "scryr_schema") {
        return validate_version(
            &text_rows(c.query(VERSION, ()).await.map_err(|e| e.to_string())?).await?,
        );
    }
    if !tables.is_empty() {
        return Err(incompatible());
    }
    let tx = c
        .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
        .await
        .map_err(|e| e.to_string())?;
    let tables = text_rows(tx.query(TABLES, ()).await.map_err(|e| e.to_string())?).await?;
    if tables.iter().any(|name| name == "scryr_schema") {
        validate_version(
            &text_rows(tx.query(VERSION, ()).await.map_err(|e| e.to_string())?).await?,
        )?;
    } else {
        if !tables.is_empty() {
            return Err(incompatible());
        }
        for statement in [CREATE_VERSION, INSERT_VERSION]
            .into_iter()
            .chain(SCHEMA_STATEMENTS.iter().map(|(_, sql)| *sql))
        {
            tx.execute(statement, ()).await.map_err(|e| e.to_string())?;
        }
    }
    tx.commit().await.map_err(|e| e.to_string())
}
/// Initialize a fresh database, or reject an incompatible existing schema without changing it.
/// # Errors
/// Returns version mismatch, bootstrap, or connection errors.
pub(crate) async fn ensure_table(pool: &DatabasePool) -> Result<(), String> {
    match pool {
        DatabasePool::Sqlite(p) => ensure_sqlite_table(p).await,
        DatabasePool::Turso(d) => ensure_turso_table(d).await,
    }
}
