//! OAuth configuration and authorize URL construction.

use crate::args::LoginArgs;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

pub(super) const DEFAULT_SCOPES: &str = "profile email offline_access";
pub(super) const DEFAULT_CLERK_PUBLISHABLE_KEY: &str = "pk_live_Y2xlcmsuc2NyeXIuYXBwJA";
pub(super) const DEFAULT_CLERK_OAUTH_CLIENT_ID: &str = "y9f16XlWxvQfR659";
pub(super) const OAUTH_CALLBACK_PORT: u16 = 9789;

#[derive(Debug, Deserialize)]
struct AuthorizationServerMetadata {
    authorization_endpoint: String,
    token_endpoint: String,
}

#[derive(Debug)]
pub(super) struct OAuthConfig {
    pub(super) authorization_endpoint: String,
    pub(super) token_endpoint: String,
    pub(super) client_id: String,
    pub(super) scope: String,
}

#[derive(Debug)]
pub(super) struct PkcePair {
    pub(super) verifier: String,
    pub(super) challenge: String,
}

pub(super) async fn build_oauth_config(args: &LoginArgs) -> Result<OAuthConfig, String> {
    let publishable_key = configured_clerk_publishable_key(args);
    let client_id = configured_clerk_oauth_client_id(args);
    let frontend_api = derive_frontend_api_origin(&publishable_key)?;
    let metadata_url = format!("{frontend_api}/.well-known/oauth-authorization-server");
    let metadata = reqwest::get(&metadata_url)
        .await
        .map_err(|error| {
            format!("Failed to download Clerk OAuth metadata from {metadata_url}: {error}")
        })?
        .error_for_status()
        .map_err(|error| {
            format!("Clerk OAuth metadata request failed for {metadata_url}: {error}")
        })?
        .json::<AuthorizationServerMetadata>()
        .await
        .map_err(|error| {
            format!("Failed to parse Clerk OAuth metadata from {metadata_url}: {error}")
        })?;

    Ok(OAuthConfig {
        authorization_endpoint: metadata.authorization_endpoint,
        token_endpoint: metadata.token_endpoint,
        client_id,
        scope: if args.scopes.trim().is_empty() {
            DEFAULT_SCOPES.to_string()
        } else {
            args.scopes.trim().to_string()
        },
    })
}

pub(super) fn configured_clerk_publishable_key(args: &LoginArgs) -> String {
    non_empty_arg_or_default(
        args.clerk_publishable_key.as_ref(),
        DEFAULT_CLERK_PUBLISHABLE_KEY,
    )
}

pub(super) fn configured_clerk_oauth_client_id(args: &LoginArgs) -> String {
    non_empty_arg_or_default(args.oauth_client_id.as_ref(), DEFAULT_CLERK_OAUTH_CLIENT_ID)
}

fn non_empty_arg_or_default(value: Option<&String>, default: &str) -> String {
    value
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(default)
        .to_string()
}

pub(super) fn derive_frontend_api_origin(publishable_key: &str) -> Result<String, String> {
    let encoded = publishable_key
        .rsplit('_')
        .next()
        .ok_or_else(|| "Invalid Clerk publishable key format.".to_string())?
        .trim_end_matches('$');

    let decoded_bytes = STANDARD
        .decode(encoded)
        .or_else(|_| URL_SAFE_NO_PAD.decode(encoded))
        .map_err(|error| format!("Failed to decode Clerk publishable key hostname: {error}"))?;
    let hostname = String::from_utf8(decoded_bytes)
        .map_err(|error| format!("Clerk publishable key hostname was not valid UTF-8: {error}"))?;
    let hostname = hostname.trim_end_matches('$');

    Ok(format!("https://{hostname}"))
}

pub(super) fn build_authorize_url(
    config: &OAuthConfig,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> Result<String, String> {
    let mut authorize_url = Url::parse(&config.authorization_endpoint)
        .map_err(|error| format!("Invalid Clerk authorization endpoint: {error}"))?;
    authorize_url
        .query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", &config.scope)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256");

    Ok(authorize_url.into())
}

pub(super) fn generate_pkce_pair() -> PkcePair {
    let verifier = random_base64_url(32);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    PkcePair {
        verifier,
        challenge,
    }
}

pub(super) fn random_base64_url(byte_count: usize) -> String {
    let mut bytes = vec![0_u8; byte_count];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub(super) fn default_scope() -> String {
    DEFAULT_SCOPES.to_string()
}
