//! JWT claim decoding helpers.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;

/// JWT claims decoded from the CLI access token.
#[derive(Debug, Deserialize)]
pub(super) struct JwtClaims {
    /// Subject/user identifier.
    pub(super) sub: String,
    /// Token expiration time.
    #[serde(default)]
    pub(super) exp: Option<u64>,
    /// Token issuer.
    #[serde(default)]
    pub(super) iss: Option<String>,
    /// Authorized party/client id.
    #[serde(default)]
    pub(super) azp: Option<String>,
    /// Granted scope string.
    #[serde(default)]
    pub(super) scope: Option<String>,
}

/// Decode JWT claims without verifying the signature.
pub(super) fn decode_jwt_claims(token: &str) -> Result<JwtClaims, String> {
    let mut parts = token.split('.');
    let _header = parts
        .next()
        .ok_or_else(|| "JWT was missing a header segment.".to_string())?;
    let payload = parts
        .next()
        .ok_or_else(|| "JWT was missing a payload segment.".to_string())?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| format!("Failed to decode JWT payload: {error}"))?;

    serde_json::from_slice(&payload_bytes)
        .map_err(|error| format!("Failed to parse JWT payload JSON: {error}"))
}

#[cfg(test)]
mod tests {
    use super::decode_jwt_claims;

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    #[test]
    fn decode_jwt_claims_reads_subject() -> TestResult {
        let token = "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ1c2VyXzEyMyIsInNjb3BlIjoib3BlbmlkIn0.";
        let claims = decode_jwt_claims(token).map_err(std::io::Error::other)?;

        assert_eq!(claims.sub, "user_123");
        assert_eq!(claims.scope.as_deref(), Some("openid"));
        Ok(())
    }
}
