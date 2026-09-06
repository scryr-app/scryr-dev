//! Clerk OAuth token exchange and refresh.

use super::StoredAuth;
use super::oauth_config::{OAuthConfig, default_scope};
use super::session::unix_timestamp_now;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct TokenResponse {
    pub(super) access_token: String,
    #[serde(default)]
    pub(super) refresh_token: Option<String>,
    pub(super) expires_in: u64,
    #[serde(default = "default_scope")]
    pub(super) scope: String,
}

pub(super) async fn exchange_authorization_code(
    client: &reqwest::Client,
    config: &OAuthConfig,
    redirect_uri: &str,
    code_verifier: &str,
    code: &str,
) -> Result<TokenResponse, String> {
    exchange_token(
        client,
        &config.token_endpoint,
        [
            ("grant_type", "authorization_code"),
            ("client_id", config.client_id.as_str()),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
            ("code", code),
        ],
    )
    .await
}

pub(super) async fn refresh_access_token(
    client: &reqwest::Client,
    stored: &StoredAuth,
) -> Result<StoredAuth, String> {
    let token_response = exchange_token(
        client,
        &stored.token_endpoint,
        [
            ("grant_type", "refresh_token"),
            ("client_id", stored.client_id.as_str()),
            ("refresh_token", stored.refresh_token.as_str()),
        ],
    )
    .await?;

    Ok(StoredAuth {
        access_token: token_response.access_token,
        refresh_token: token_response
            .refresh_token
            .unwrap_or_else(|| stored.refresh_token.clone()),
        expires_at_epoch_seconds: unix_timestamp_now() + token_response.expires_in,
        scope: token_response.scope,
        token_endpoint: stored.token_endpoint.clone(),
        authorization_endpoint: stored.authorization_endpoint.clone(),
        client_id: stored.client_id.clone(),
    })
}

async fn exchange_token<const N: usize>(
    client: &reqwest::Client,
    token_endpoint: &str,
    form: [(&str, &str); N],
) -> Result<TokenResponse, String> {
    let form = form.into_iter().collect::<Vec<_>>();
    let response = client
        .post(token_endpoint)
        .form(&form)
        .send()
        .await
        .map_err(|error| {
            format!("Failed to call Clerk token endpoint {token_endpoint}: {error}")
        })?;
    let status = response.status();
    let body = response.text().await.map_err(|error| {
        format!("Failed to read Clerk token response from {token_endpoint}: {error}")
    })?;

    if status != StatusCode::OK {
        return Err(format!(
            "Clerk token endpoint {token_endpoint} returned HTTP {status}: {body}"
        ));
    }

    serde_json::from_str(&body).map_err(|error| {
        format!("Failed to parse Clerk token response from {token_endpoint}: {error}; body: {body}")
    })
}
