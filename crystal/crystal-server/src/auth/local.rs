//! Zero-config local development authentication.

use super::models::{AuthOrganizationContext, AuthenticatedPrincipal};

const LOCAL_USER_ID: &str = "local-dev-user";
const LOCAL_ORG_ID: &str = "local-dev-org";
const LOCAL_ORG_SLUG: &str = "local-dev";
const LOCAL_ORG_ROLE: &str = "org:admin";
const LOCAL_ORG_PERMISSION: &str = "scryr:maps:write";

impl AuthenticatedPrincipal {
    /// Build the zero-config local development principal.
    pub(super) fn local_dev() -> Self {
        Self::new(
            LOCAL_USER_ID.to_string(),
            AuthOrganizationContext {
                id: LOCAL_ORG_ID.to_string(),
                slug: Some(LOCAL_ORG_SLUG.to_string()),
                role: Some(LOCAL_ORG_ROLE.to_string()),
                permissions: vec![LOCAL_ORG_PERMISSION.to_string()],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::models::AuthenticatedRequest;
    use super::AuthenticatedPrincipal;

    #[test]
    fn local_authentication_has_writable_manifest_context() -> Result<(), String> {
        let auth = AuthenticatedRequest::Local(AuthenticatedPrincipal::local_dev());

        let context = auth.manifest_request_context()?;

        assert_eq!(auth.user_id(), "local-dev-user");
        assert_eq!(context.clerk_user_id, "local-dev-user");
        assert_eq!(context.clerk_org_id, "local-dev-org");
        assert_eq!(context.clerk_org_slug.as_deref(), Some("local-dev"));
        assert!(context.can_write_generated_manifests());
        Ok(())
    }
}
