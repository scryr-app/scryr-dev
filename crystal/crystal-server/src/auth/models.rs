//! Provider-neutral authenticated request models.

use super::clerk::active_organization_from_jwt;
use crystal_core::manifest::ManifestRequestContext;

/// Authenticated request identity added to async-graphql request context.
#[derive(Clone, Debug)]
pub(crate) enum AuthenticatedRequest {
    /// Request was authenticated with the zero-config local development principal.
    Local(AuthenticatedPrincipal),
    /// Request was authenticated with a Clerk session token.
    Clerk(clerk_rs::validators::authorizer::ClerkJwt),
}

impl AuthenticatedRequest {
    /// Returns the normalized authenticated user id.
    pub(crate) const fn user_id(&self) -> &str {
        match self {
            Self::Local(principal) => principal.user_id.as_str(),
            Self::Clerk(jwt) => jwt.sub.as_str(),
        }
    }

    /// Returns the selected auth provider name.
    pub(crate) const fn provider(&self) -> &'static str {
        match self {
            Self::Local(_) => "local",
            Self::Clerk(_) => "clerk",
        }
    }

    /// Returns the active organization context for the request.
    pub(crate) fn active_organization(&self) -> Option<AuthOrganizationContext> {
        match self {
            Self::Local(principal) => Some(principal.organization.clone()),
            Self::Clerk(jwt) => active_organization_from_jwt(jwt),
        }
    }

    /// Returns the generated-manifest request context for resolvers that need tenant scoping.
    pub(crate) fn manifest_request_context(&self) -> Result<ManifestRequestContext, String> {
        let organization = self.active_organization().ok_or_else(|| {
            format!(
                "request authenticated with {} must include an active organization before accessing Scryr maps",
                self.provider()
            )
        })?;

        Ok(ManifestRequestContext {
            clerk_user_id: self.user_id().to_string(),
            clerk_org_id: organization.id,
            clerk_org_slug: organization.slug,
            clerk_org_role: organization.role,
            clerk_org_permissions: organization.permissions,
        })
    }
}

/// Provider-neutral authenticated principal used by local mode and future auth providers.
#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedPrincipal {
    /// Stable user id from the selected auth provider.
    pub(super) user_id: String,
    /// Active organization/tenant context for the request.
    pub(super) organization: AuthOrganizationContext,
}

impl AuthenticatedPrincipal {
    /// Build a provider-neutral principal.
    pub(super) const fn new(user_id: String, organization: AuthOrganizationContext) -> Self {
        Self {
            user_id,
            organization,
        }
    }
}

/// Active organization attributes normalized across auth provider claim versions.
#[derive(Clone, Debug)]
pub(crate) struct AuthOrganizationContext {
    /// Provider organization id.
    pub(crate) id: String,
    /// Organization slug, when present in the provider token.
    pub(crate) slug: Option<String>,
    /// Organization role, when present in the provider token.
    pub(crate) role: Option<String>,
    /// Organization custom permissions present in the provider token.
    pub(crate) permissions: Vec<String>,
}
