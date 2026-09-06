//! Database URL parsing and pool setup for generated manifest persistence.

use super::models::DatabasePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

const SQLITE_PATH_ENV: &str = "SCRYR_SQLITE_PATH";
const TURSO_DATABASE_URL_ENV: &str = "TURSO_DATABASE_URL";
const TURSO_AUTH_TOKEN_ENV: &str = "TURSO_AUTH_TOKEN";
const LIBSQL_AUTH_TOKEN_ENV: &str = "LIBSQL_AUTH_TOKEN";

/// Return whether a database URL names Postgres.
fn is_postgres_url(database_url: &str) -> bool {
    database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")
}

/// Return whether a database URL names a remote Turso/libSQL database.
fn is_turso_url(database_url: &str) -> bool {
    database_url.starts_with("libsql://")
        || database_url.starts_with("https://")
        || database_url.starts_with("wss://")
}

/// Return whether a database URL names `SQLite`.
fn is_sqlite_url(database_url: &str) -> bool {
    database_url.starts_with("sqlite:")
}

/// Return the default `SQLite` database path for local development.
fn default_sqlite_path() -> PathBuf {
    env::var_os(SQLITE_PATH_ENV)
        .map_or_else(|| PathBuf::from(".scryr").join("scryr.db"), PathBuf::from)
}

/// Return the configured local `SQLite` database path when storage uses a file.
#[must_use]
pub fn sqlite_path_from_env() -> Option<PathBuf> {
    if env::var(TURSO_DATABASE_URL_ENV)
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return None;
    }

    let database_url = env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty());

    let Some(database_url) = database_url else {
        return Some(default_sqlite_path());
    };

    if is_postgres_url(&database_url) || is_turso_url(&database_url) {
        return None;
    }

    if is_sqlite_url(&database_url) {
        return sqlite_path_from_url(&database_url);
    }

    Some(PathBuf::from(database_url))
}

/// Return the file path portion of a file-backed `SQLite` connection URL.
pub(super) fn sqlite_path_from_url(database_url: &str) -> Option<PathBuf> {
    let url_without_scheme = database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let (database, params) = url_without_scheme
        .split_once('?')
        .map_or((url_without_scheme, None), |(database, params)| {
            (database, Some(params))
        });

    if database == ":memory:" || params_has_sqlite_memory_mode(params) || database.is_empty() {
        return None;
    }

    Some(PathBuf::from(percent_decode_lossy(database)))
}

/// Return whether `SQLite` URL query parameters request an in-memory database.
fn params_has_sqlite_memory_mode(params: Option<&str>) -> bool {
    params.is_some_and(|params| {
        params
            .split('&')
            .filter_map(|part| part.split_once('='))
            .any(|(key, value)| key == "mode" && value == "memory")
    })
}

/// Percent-decode a `SQLite` URL path using lossy UTF-8 conversion.
fn percent_decode_lossy(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2))
            && let Some(decoded_byte) = decode_percent_byte(*high, *low)
        {
            decoded.push(decoded_byte);
            index += 3;
            continue;
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

/// Decode two hexadecimal characters into one byte.
fn decode_percent_byte(high: u8, low: u8) -> Option<u8> {
    Some(hex_value(high)? * 16 + hex_value(low)?)
}

/// Return the numeric value for an ASCII hexadecimal byte.
const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

/// Connect to a `SQLite` database file, creating the file and its parent directory.
///
/// # Errors
///
/// Returns an error if the path cannot be prepared or `SQLite` cannot connect.
pub(crate) async fn connect_sqlite_path(path: &Path) -> Result<DatabasePool, String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Failed to create SQLite directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let options = SqliteConnectOptions::from_str(&path.to_string_lossy())
        .map_err(|error| {
            format!(
                "Failed to configure SQLite path {}: {error}",
                path.display()
            )
        })?
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| {
            format!(
                "Failed to connect to SQLite database {}: {error}",
                path.display()
            )
        })?;

    Ok(DatabasePool::Sqlite(pool))
}

/// Connect to a remote Turso/libSQL database.
///
/// # Errors
///
/// Returns an error if the remote URL cannot be opened.
async fn connect_turso_url(database_url: &str, auth_token: &str) -> Result<DatabasePool, String> {
    let database = libsql::Builder::new_remote(database_url.to_string(), auth_token.to_string())
        .build()
        .await
        .map_err(|error| format!("Failed to connect to Turso database: {error}"))?;

    Ok(DatabasePool::Turso(Arc::new(database)))
}

/// Read the configured Turso auth token from environment.
fn turso_auth_token_from_env() -> Result<String, String> {
    env::var(TURSO_AUTH_TOKEN_ENV)
        .or_else(|_| env::var(LIBSQL_AUTH_TOKEN_ENV))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string())
        .ok_or_else(|| {
            format!("Turso storage requires {TURSO_AUTH_TOKEN_ENV} or {LIBSQL_AUTH_TOKEN_ENV}")
        })
}

/// Return the unsupported Postgres configuration error.
fn postgres_removed_error() -> String {
    "Postgres storage is no longer supported. Use TURSO_DATABASE_URL with TURSO_AUTH_TOKEN, or configure local SQLite with DATABASE_URL=sqlite:... or SCRYR_SQLITE_PATH.".to_string()
}

/// Connect to storage from environment, defaulting to local `SQLite` when unset.
///
/// # Errors
///
/// Returns an error if the configured database cannot be opened.
pub(crate) async fn connect_from_env() -> Result<DatabasePool, String> {
    if let Some(turso_database_url) = env::var(TURSO_DATABASE_URL_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        let auth_token = turso_auth_token_from_env()?;
        return connect_turso_url(turso_database_url.trim(), &auth_token).await;
    }

    let database_url = env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty());

    let Some(database_url) = database_url else {
        return connect_sqlite_path(&default_sqlite_path()).await;
    };

    if is_postgres_url(&database_url) {
        return Err(postgres_removed_error());
    }

    if is_turso_url(&database_url) {
        let auth_token = turso_auth_token_from_env()?;
        return connect_turso_url(database_url.trim(), &auth_token).await;
    }

    if is_sqlite_url(&database_url) {
        let options = SqliteConnectOptions::from_str(&database_url)
            .map_err(|error| format!("Failed to parse SQLite DATABASE_URL: {error}"))?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|error| format!("Failed to connect to SQLite DATABASE_URL: {error}"))?;
        return Ok(DatabasePool::Sqlite(pool));
    }

    connect_sqlite_path(Path::new(&database_url)).await
}
