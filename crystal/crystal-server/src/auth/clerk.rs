//! Clerk authentication claim and membership helpers.

use super::models::{AuthOrganizationContext, AuthenticatedRequest};
use crate::state::AppState;
use clerk_rs::{apis::users_api::User as ClerkUserApi, validators::authorizer::ClerkJwt};
use crystal_core::manifest::ManifestRequestContext;
use serde_json::{Map, Value};

/// Verify a CLI organization selector against Clerk membership, or infer a sole membership.
pub(super) async fn verified_manifest_request_context(
    auth: &AuthenticatedRequest,
    state: &AppState,
    requested_org_id: Option<&str>,
) -> Result<ManifestRequestContext, String> {
    let Some(clerk_client) = &state.clerk_client else {
        return Err(
            "CLERK_SECRET_KEY is required to verify CLI organization membership".to_string(),
        );
    };

    let mut verified_contexts = Vec::new();
    let mut offset = 0_u64;
    loop {
        let memberships = ClerkUserApi::users_get_organization_memberships(
            clerk_client,
            auth.user_id(),
            Some(100),
            Some(offset),
        )
        .await
        .map_err(|error| {
            format!(
                "Failed to verify Clerk organization membership for {}: {error}",
                auth.user_id()
            )
        })?;
        let returned_count = memberships.data.len() as u64;

        for membership in memberships.data {
            let Some(organization) = membership.organization.as_deref() else {
                continue;
            };
            let context = ManifestRequestContext {
                clerk_user_id: auth.user_id().to_string(),
                clerk_org_id: organization.id.clone(),
                clerk_org_slug: non_empty_string(Some(organization.slug.as_str())),
                clerk_org_role: membership.role,
                clerk_org_permissions: membership.permissions.unwrap_or_default(),
            };
            if requested_org_id == Some(context.clerk_org_id.as_str()) {
                return Ok(context);
            }
            verified_contexts.push(context);
        }

        let total_count = u64::try_from(memberships.total_count)
            .map_err(|_| "Clerk organization membership response had negative total_count")?;
        if returned_count == 0 || offset + returned_count >= total_count {
            break;
        }
        offset += returned_count;
    }

    if let Some(requested_org_id) = requested_org_id {
        return Err(format!(
            "authenticated Clerk user is not a member of organization {requested_org_id}"
        ));
    }

    match verified_contexts.as_slice() {
        [context] => Ok(context.clone()),
        [] => Err("authenticated Clerk user is not a member of any organization".to_string()),
        contexts => Err(format!(
            "authenticated Clerk user belongs to multiple organizations; pass --clerk-org-id or set SCRYR_CLERK_ORG_ID. Available organization ids: {}",
            contexts
                .iter()
                .map(|context| context.clerk_org_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Extract active organization data from verified Clerk JWT claims.
pub(super) fn active_organization_from_jwt(jwt: &ClerkJwt) -> Option<AuthOrganizationContext> {
    if let Some(org) = &jwt.org {
        return Some(AuthOrganizationContext {
            id: org.id.clone(),
            slug: non_empty_string(Some(org.slug.as_str())),
            role: non_empty_string(Some(org.role.as_str())),
            permissions: org.permissions.clone(),
        });
    }

    compact_organization_from_claims(&jwt.other)
        .or_else(|| top_level_organization_from_claims(&jwt.other))
}

/// Extract Clerk Core 2 compact `o` claim organization data.
fn compact_organization_from_claims(
    claims: &Map<String, Value>,
) -> Option<AuthOrganizationContext> {
    let object = claims.get("o")?.as_object()?;
    let id = non_empty_json_string(object.get("id"))?;

    Some(AuthOrganizationContext {
        id,
        slug: non_empty_json_string(object.get("slg")),
        role: non_empty_json_string(object.get("rol")),
        permissions: permissions_from_claim(object.get("per")),
    })
}

/// Extract older top-level active organization claim data.
fn top_level_organization_from_claims(
    claims: &Map<String, Value>,
) -> Option<AuthOrganizationContext> {
    let id = non_empty_json_string(claims.get("org_id"))?;

    Some(AuthOrganizationContext {
        id,
        slug: non_empty_json_string(claims.get("org_slug")),
        role: non_empty_json_string(claims.get("org_role")),
        permissions: permissions_from_claim(claims.get("org_permissions")),
    })
}

/// Normalize a JSON claim value into an optional non-empty string.
fn non_empty_json_string(value: Option<&Value>) -> Option<String> {
    non_empty_string(value.and_then(Value::as_str))
}

/// Normalize optional text into a non-empty owned string.
fn non_empty_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// Decode Clerk permission claims from either JSON arrays or compact comma-separated strings.
fn permissions_from_claim(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|value| non_empty_string(Some(value)))
            .collect(),
        Some(Value::String(value)) => value
            .split(',')
            .filter_map(|value| non_empty_string(Some(value)))
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::active_organization_from_jwt;
    use clerk_rs::validators::authorizer::ClerkJwt;
    use serde_json::{Map, json};

    fn jwt_with_other_claims(other: Map<String, serde_json::Value>) -> ClerkJwt {
        ClerkJwt {
            azp: None,
            exp: 2,
            iat: 1,
            iss: "issuer".to_string(),
            nbf: 1,
            sid: Some("session".to_string()),
            sub: "user_123".to_string(),
            act: None,
            org: None,
            other,
        }
    }

    #[test]
    fn active_organization_supports_compact_clerk_claim() -> Result<(), String> {
        let other = json!({
            "o": {
                "id": "org_123",
                "slg": "acme",
                "rol": "admin",
                "per": "maps:read,maps:write"
            }
        })
        .as_object()
        .cloned()
        .ok_or_else(|| "test JSON should be an object".to_string())?;
        let jwt = jwt_with_other_claims(other);

        let organization = active_organization_from_jwt(&jwt)
            .ok_or_else(|| "expected active organization".to_string())?;

        assert_eq!(organization.id, "org_123");
        assert_eq!(organization.slug.as_deref(), Some("acme"));
        assert_eq!(organization.role.as_deref(), Some("admin"));
        assert_eq!(
            organization.permissions,
            vec!["maps:read".to_string(), "maps:write".to_string()]
        );
        Ok(())
    }

    #[test]
    fn active_organization_supports_top_level_clerk_claims() -> Result<(), String> {
        let other = json!({
            "org_id": "org_456",
            "org_slug": "globex",
            "org_role": "org:member",
            "org_permissions": ["maps:read"]
        })
        .as_object()
        .cloned()
        .ok_or_else(|| "test JSON should be an object".to_string())?;
        let jwt = jwt_with_other_claims(other);

        let organization = active_organization_from_jwt(&jwt)
            .ok_or_else(|| "expected active organization".to_string())?;

        assert_eq!(organization.id, "org_456");
        assert_eq!(organization.slug.as_deref(), Some("globex"));
        assert_eq!(organization.role.as_deref(), Some("org:member"));
        assert_eq!(organization.permissions, vec!["maps:read".to_string()]);
        Ok(())
    }
}
