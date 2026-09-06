//! HTTP and GraphQL request authentication plumbing.

use super::clerk::verified_manifest_request_context;
use super::models::{AuthenticatedPrincipal, AuthenticatedRequest};
use crate::state::{AppState, AuthMode};
use actix_web::HttpRequest;
use async_graphql::Context;
use clerk_rs::validators::authorizer::{ClerkError, ClerkRequest};
use crystal_core::manifest::ManifestRequestContext;

/// Header used by non-browser clients to request a Clerk organization scope.
pub(crate) const SCRYR_CLERK_ORG_ID_HEADER: &str = "x-scryr-clerk-org-id";

struct HttpRequestAdapter<'a>(&'a HttpRequest);

impl ClerkRequest for HttpRequestAdapter<'_> {
    fn get_header(&self, key: &str) -> Option<String> {
        self.0
            .headers()
            .get(key)
            .and_then(|value| value.to_str().ok())
            .map(std::string::ToString::to_string)
    }

    fn get_cookie(&self, key: &str) -> Option<String> {
        self.0.cookie(key).map(|cookie| cookie.value().to_string())
    }
}

/// Resolve the manifest tenant context for this request.
pub(crate) async fn resolve_manifest_request_context(
    request: &HttpRequest,
    auth: &AuthenticatedRequest,
    state: &AppState,
) -> Result<ManifestRequestContext, String> {
    if let Ok(context) = auth.manifest_request_context() {
        return Ok(context);
    }

    verified_manifest_request_context(auth, state, requested_clerk_org_id(request).as_deref()).await
}

/// Return whether a request includes an explicit Clerk organization id selector.
pub(crate) fn has_requested_clerk_org_id(request: &HttpRequest) -> bool {
    requested_clerk_org_id(request).is_some()
}

/// Extract the explicit Clerk organization id selector header.
fn requested_clerk_org_id(request: &HttpRequest) -> Option<String> {
    request
        .headers()
        .get(SCRYR_CLERK_ORG_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// Authenticate an incoming GraphQL request with the configured provider.
pub(crate) async fn authenticate_request(
    request: &HttpRequest,
    state: &AppState,
) -> Result<AuthenticatedRequest, String> {
    match state.auth_mode {
        AuthMode::Local => {
            return Ok(AuthenticatedRequest::Local(
                AuthenticatedPrincipal::local_dev(),
            ));
        }
        AuthMode::Clerk => {}
    }

    let Some(authorizer) = &state.clerk_authorizer else {
        return Err(
            "Clerk authentication is not configured on the server. Set CLERK_SECRET_KEY or run with AUTH_MODE=local for local development."
                .to_string(),
        );
    };

    authorizer
        .authorize(&HttpRequestAdapter(request))
        .await
        .map(AuthenticatedRequest::Clerk)
        .map_err(|error| match error {
            ClerkError::Unauthorized(message) | ClerkError::InternalServerError(message) => message,
        })
}

/// Load the authenticated request from async-graphql context.
pub(crate) fn require_authenticated_request<'a>(
    ctx: &'a Context<'a>,
) -> async_graphql::Result<&'a AuthenticatedRequest> {
    ctx.data::<AuthenticatedRequest>()
        .map_err(|_| async_graphql::Error::new("request is missing authenticated identity"))
}
