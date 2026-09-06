//! Auth session commands and stored-token lifecycle.

use super::StoredAuth;
use super::browser::open_browser;
use super::callback::{CallbackPayload, spawn_callback_listener};
use super::jwt::decode_jwt_claims;
use super::oauth_config::{
    OAUTH_CALLBACK_PORT, OAuthConfig, build_authorize_url, build_oauth_config, generate_pkce_pair,
    random_base64_url,
};
use super::storage::{auth_file_path, persist_auth_to_file, read_auth_from_file, remove_auth_file};
use super::token_exchange::{exchange_authorization_code, refresh_access_token};
use crate::args::LoginArgs;
use std::net::TcpListener;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CALLBACK_TIMEOUT_SECS: u64 = 5 * 60;
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(CALLBACK_TIMEOUT_SECS);
const TOKEN_EXPIRY_SKEW_SECS: u64 = 60;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct WhoAmIOutput {
    pub(super) sub: String,
    pub(super) iss: Option<String>,
    pub(super) azp: Option<String>,
    pub(super) scope: String,
    pub(super) exp: Option<u64>,
}

/// Complete an interactive Clerk OAuth login and persist the refresh state.
pub(crate) async fn login(args: &LoginArgs) -> Result<(), String> {
    let auth_file = auth_file_path()?;
    login_with_auth_file(args, &auth_file).await
}

pub(super) async fn login_with_auth_file(args: &LoginArgs, auth_file: &Path) -> Result<(), String> {
    let oauth_config = build_oauth_config(args).await?;
    let pkce = generate_pkce_pair();
    let state = random_base64_url(24);
    let listener = TcpListener::bind(("127.0.0.1", OAUTH_CALLBACK_PORT)).map_err(|error| {
        format!(
            "Failed to bind local OAuth callback listener on http://127.0.0.1:{OAUTH_CALLBACK_PORT}/oauth/callback: {error}"
        )
    })?;
    let redirect_uri = format!("http://127.0.0.1:{OAUTH_CALLBACK_PORT}/oauth/callback");
    let authorize_url = build_authorize_url(&oauth_config, &redirect_uri, &state, &pkce.challenge)?;
    let callback_receiver = spawn_callback_listener(listener);

    println!("OAuth authorize URL: {authorize_url}");
    open_browser(&authorize_url)?;
    println!("Opening browser for Clerk sign-in...");

    let callback = callback_receiver
        .recv_timeout(CALLBACK_TIMEOUT)
        .map_err(|error| {
            format!(
                "Timed out waiting for the OAuth callback after {} seconds: {error}",
                CALLBACK_TIMEOUT.as_secs()
            )
        })??;

    let client = reqwest::Client::new();
    let stored_auth = complete_login(
        &client,
        oauth_config,
        &redirect_uri,
        &pkce.verifier,
        callback,
        &state,
        auth_file,
    )
    .await?;
    let claims = decode_jwt_claims(&stored_auth.access_token)?;
    println!("Authenticated as {}", claims.sub);
    Ok(())
}

/// Remove the locally cached CLI auth state.
pub(crate) fn logout() -> Result<(), String> {
    let auth_file = auth_file_path()?;
    remove_auth_file(&auth_file)
}

/// Print the current authenticated subject after refreshing the access token if needed.
pub(crate) async fn whoami() -> Result<(), String> {
    let auth_file = auth_file_path()?;
    let summary = whoami_summary_from_auth_file(&auth_file).await?;

    println!("sub: {}", summary.sub);
    if let Some(issuer) = summary.iss {
        println!("iss: {issuer}");
    }
    if let Some(audience) = summary.azp {
        println!("azp: {audience}");
    }
    println!("scope: {}", summary.scope);
    if let Some(exp) = summary.exp {
        println!("exp: {exp}");
    }

    Ok(())
}

/// Load a usable access token, refreshing it if the cached token is stale.
pub(crate) async fn access_token() -> Result<String, String> {
    let auth_file = auth_file_path()?;
    load_or_refresh_auth_from_file(&auth_file)
        .await
        .map(|auth| auth.access_token)
}

pub(super) async fn load_or_refresh_auth_from_file(auth_file: &Path) -> Result<StoredAuth, String> {
    let stored = read_auth_from_file(auth_file)?;

    if !token_is_stale(stored.expires_at_epoch_seconds) {
        return Ok(stored);
    }

    let client = reqwest::Client::new();
    let refreshed = refresh_access_token(&client, &stored).await?;
    persist_auth_to_file(auth_file, &refreshed)?;
    Ok(refreshed)
}

pub(super) async fn complete_login(
    client: &reqwest::Client,
    oauth_config: OAuthConfig,
    redirect_uri: &str,
    pkce_verifier: &str,
    callback: CallbackPayload,
    expected_state: &str,
    auth_file: &Path,
) -> Result<StoredAuth, String> {
    if callback.state != expected_state {
        return Err("OAuth callback state mismatch. Please retry `scryr auth login`.".to_string());
    }

    let token_response = exchange_authorization_code(
        client,
        &oauth_config,
        redirect_uri,
        pkce_verifier,
        &callback.code,
    )
    .await?;
    let refresh_token = token_response.refresh_token.ok_or_else(|| {
        "Clerk OAuth response did not include a refresh token. Ensure the OAuth app allows offline access."
            .to_string()
    })?;
    let stored_auth = StoredAuth {
        access_token: token_response.access_token,
        refresh_token,
        expires_at_epoch_seconds: unix_timestamp_now() + token_response.expires_in,
        scope: token_response.scope,
        token_endpoint: oauth_config.token_endpoint,
        authorization_endpoint: oauth_config.authorization_endpoint,
        client_id: oauth_config.client_id,
    };

    persist_auth_to_file(auth_file, &stored_auth)?;
    Ok(stored_auth)
}

pub(super) async fn whoami_summary_from_auth_file(
    auth_file: &Path,
) -> Result<WhoAmIOutput, String> {
    let auth = load_or_refresh_auth_from_file(auth_file).await?;
    let claims = decode_jwt_claims(&auth.access_token)?;

    Ok(WhoAmIOutput {
        sub: claims.sub,
        iss: claims.iss,
        azp: claims.azp,
        scope: claims.scope.unwrap_or(auth.scope),
        exp: claims.exp,
    })
}

fn token_is_stale(expires_at_epoch_seconds: u64) -> bool {
    expires_at_epoch_seconds <= unix_timestamp_now() + TOKEN_EXPIRY_SKEW_SECS
}

pub(super) fn unix_timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
