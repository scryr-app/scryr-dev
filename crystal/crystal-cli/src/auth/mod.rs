#![allow(clippy::missing_docs_in_private_items, clippy::redundant_pub_crate)]

pub(crate) mod browser;
mod callback;
mod jwt;
pub(crate) mod oauth_config;
pub(crate) mod session;
mod storage;
pub(crate) mod token_exchange;

use serde::{Deserialize, Serialize};

/// Stored OAuth state for the Scryr CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct StoredAuth {
    access_token: String,
    refresh_token: String,
    expires_at_epoch_seconds: u64,
    scope: String,
    token_endpoint: String,
    authorization_endpoint: String,
    client_id: String,
}

pub(crate) use session::{access_token, login, logout, whoami};

#[cfg(test)]
mod tests {
    use super::callback::CallbackPayload;
    use super::oauth_config::{
        DEFAULT_CLERK_OAUTH_CLIENT_ID, DEFAULT_CLERK_PUBLISHABLE_KEY, OAuthConfig,
        build_authorize_url, configured_clerk_oauth_client_id, configured_clerk_publishable_key,
        derive_frontend_api_origin,
    };
    use super::session::{WhoAmIOutput, complete_login, whoami_summary_from_auth_file};
    use super::storage::{persist_auth_to_file, remove_auth_file};
    use crate::args::LoginArgs;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
    type SeenRequests = Arc<Mutex<Vec<HashMap<String, String>>>>;

    fn test_auth_path(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("scryr-cli-auth-{label}-{unique}.json"))
    }

    fn sample_jwt(sub: &str, scope: &str) -> Result<String, serde_json::Error> {
        let header = "eyJhbGciOiJub25lIn0";
        let payload = serde_json::to_vec(&json!({
            "sub": sub,
            "scope": scope,
            "iss": "https://issuer.example",
            "azp": "client_123",
            "exp": 4_102_444_800_u64,
        }))?;
        let payload = URL_SAFE_NO_PAD.encode(payload);

        Ok(format!("{header}.{payload}."))
    }

    fn spawn_token_server(
        responses: Vec<serde_json::Value>,
    ) -> std::io::Result<(String, SeenRequests)> {
        let seen_requests = Arc::new(Mutex::new(Vec::new()));
        let seen_requests_for_thread = Arc::clone(&seen_requests);
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        thread::spawn(move || {
            for response in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buffer = vec![0_u8; 8192];
                let Ok(bytes_read) = stream.read(&mut buffer) else {
                    return;
                };
                let Ok(request) = String::from_utf8(buffer[..bytes_read].to_vec()) else {
                    return;
                };
                let Some(body) = request.split("\r\n\r\n").nth(1) else {
                    return;
                };
                let parsed_form = url::form_urlencoded::parse(body.trim().as_bytes())
                    .into_owned()
                    .collect::<HashMap<String, String>>();
                let Ok(mut seen_requests) = seen_requests_for_thread.lock() else {
                    return;
                };
                seen_requests.push(parsed_form);
                drop(seen_requests);

                let Ok(payload) = serde_json::to_string(&response) else {
                    return;
                };
                let http_response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                if stream.write_all(http_response.as_bytes()).is_err() {
                    return;
                }
            }
        });

        Ok((format!("http://{address}/oauth/token"), seen_requests))
    }

    #[test]
    fn derive_frontend_api_origin_decodes_publishable_key() -> TestResult {
        let origin = derive_frontend_api_origin(
            "pk_test_Y29oZXJlbnQtbWFydGVuLTUxLmNsZXJrLmFjY291bnRzLmRldiQ",
        )
        .map_err(std::io::Error::other)?;

        assert_eq!(origin, "https://coherent-marten-51.clerk.accounts.dev");
        Ok(())
    }

    #[test]
    fn login_config_uses_embedded_defaults_with_optional_overrides() {
        let default_args = LoginArgs {
            clerk_publishable_key: None,
            oauth_client_id: None,
            scopes: "profile email".to_string(),
        };
        assert_eq!(
            configured_clerk_publishable_key(&default_args),
            DEFAULT_CLERK_PUBLISHABLE_KEY
        );
        assert_eq!(
            configured_clerk_oauth_client_id(&default_args),
            DEFAULT_CLERK_OAUTH_CLIENT_ID
        );

        let override_args = LoginArgs {
            clerk_publishable_key: Some("pk_test_override".to_string()),
            oauth_client_id: Some("client_override".to_string()),
            scopes: "profile email".to_string(),
        };
        assert_eq!(
            configured_clerk_publishable_key(&override_args),
            "pk_test_override"
        );
        assert_eq!(
            configured_clerk_oauth_client_id(&override_args),
            "client_override"
        );
    }

    #[test]
    fn build_authorize_url_includes_pkce_parameters() -> TestResult {
        let config = OAuthConfig {
            authorization_endpoint: "https://example.clerk.accounts.dev/oauth/authorize"
                .to_string(),
            token_endpoint: "https://example.clerk.accounts.dev/oauth/token".to_string(),
            client_id: "client_123".to_string(),
            scope: "openid profile email offline_access".to_string(),
        };
        let url = build_authorize_url(
            &config,
            "http://127.0.0.1:1234/oauth/callback",
            "state-1",
            "challenge-1",
        )
        .map_err(std::io::Error::other)?;

        assert!(url.contains("client_id=client_123"));
        assert!(url.contains("code_challenge=challenge-1"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("offline_access"));
        Ok(())
    }

    #[tokio::test]
    async fn login_persists_auth_state_after_token_exchange() -> TestResult {
        let auth_path = test_auth_path("login");
        let access_token = sample_jwt("user_login", "profile email")?;
        let (token_endpoint, seen_requests) = match spawn_token_server(vec![json!({
            "access_token": access_token,
            "refresh_token": "refresh-login",
            "expires_in": 3600,
            "scope": "profile email",
        })]) {
            Ok(server) => server,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("Skipping auth integration test: {error}");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        let oauth_config = OAuthConfig {
            authorization_endpoint: "http://127.0.0.1/oauth/authorize".to_string(),
            token_endpoint: token_endpoint.clone(),
            client_id: "client_123".to_string(),
            scope: "profile email".to_string(),
        };

        let stored_auth = complete_login(
            &reqwest::Client::new(),
            oauth_config,
            "http://127.0.0.1:9789/oauth/callback",
            "pkce-verifier",
            CallbackPayload {
                code: "code-123".to_string(),
                state: "state-123".to_string(),
            },
            "state-123",
            &auth_path,
        )
        .await
        .map_err(std::io::Error::other)?;

        let persisted = fs::read_to_string(&auth_path)?;
        assert!(persisted.contains("refresh-login"));
        assert_eq!(stored_auth.refresh_token, "refresh-login");
        assert_eq!(stored_auth.client_id, "client_123");

        let request = {
            let requests = seen_requests
                .lock()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            requests
                .first()
                .cloned()
                .ok_or_else(|| std::io::Error::other("expected token exchange request"))?
        };
        assert_eq!(
            request.get("grant_type").map(String::as_str),
            Some("authorization_code")
        );
        assert_eq!(request.get("code").map(String::as_str), Some("code-123"));
        assert_eq!(
            request.get("code_verifier").map(String::as_str),
            Some("pkce-verifier")
        );

        remove_auth_file(&auth_path).map_err(std::io::Error::other)?;
        Ok(())
    }

    #[tokio::test]
    async fn whoami_refreshes_stale_tokens_from_stored_auth_state() -> TestResult {
        let auth_path = test_auth_path("whoami");
        let refreshed_token = sample_jwt("user_refresh", "openid profile")?;
        let (token_endpoint, seen_requests) = match spawn_token_server(vec![json!({
            "access_token": refreshed_token,
            "refresh_token": "refresh-next",
            "expires_in": 7200,
            "scope": "openid profile",
        })]) {
            Ok(server) => server,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("Skipping auth integration test: {error}");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };

        persist_auth_to_file(
            &auth_path,
            &super::StoredAuth {
                access_token: sample_jwt("stale_user", "profile")?,
                refresh_token: "refresh-original".to_string(),
                expires_at_epoch_seconds: 0,
                scope: "profile".to_string(),
                token_endpoint,
                authorization_endpoint: "http://127.0.0.1/oauth/authorize".to_string(),
                client_id: "client_123".to_string(),
            },
        )
        .map_err(std::io::Error::other)?;

        let summary = whoami_summary_from_auth_file(&auth_path)
            .await
            .map_err(std::io::Error::other)?;

        assert_eq!(
            summary,
            WhoAmIOutput {
                sub: "user_refresh".to_string(),
                iss: Some("https://issuer.example".to_string()),
                azp: Some("client_123".to_string()),
                scope: "openid profile".to_string(),
                exp: Some(4_102_444_800),
            }
        );

        let refreshed = fs::read_to_string(&auth_path)?;
        assert!(refreshed.contains("refresh-next"));

        let request = {
            let requests = seen_requests
                .lock()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            requests
                .first()
                .cloned()
                .ok_or_else(|| std::io::Error::other("expected refresh token request"))?
        };
        assert_eq!(
            request.get("grant_type").map(String::as_str),
            Some("refresh_token")
        );
        assert_eq!(
            request.get("refresh_token").map(String::as_str),
            Some("refresh-original")
        );

        remove_auth_file(&auth_path).map_err(std::io::Error::other)?;
        Ok(())
    }
}
