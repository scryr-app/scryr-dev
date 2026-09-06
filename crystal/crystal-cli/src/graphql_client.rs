use crystal_core::manifest::{UpsertGeneratedManifestInput, UpsertGeneratedManifestPayload};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

/// Header used by non-browser clients to request a Clerk organization scope.
const SCRYR_CLERK_ORG_ID_HEADER: &str = "x-scryr-clerk-org-id";

/// GraphQL mutation used to persist generated manifest artifacts through HTTP.
const UPSERT_GENERATED_MANIFEST_MUTATION: &str = r"
mutation UpsertGeneratedManifest($input: UpsertGeneratedManifestInput!) {
  upsertGeneratedManifest(input: $input) {
    id
  }
}
";

/// Serialized GraphQL POST body.
#[derive(Serialize)]
struct GraphqlRequestBody {
    /// GraphQL mutation document.
    query: &'static str,
    /// Variable payload sent with the mutation.
    variables: GraphqlVariables,
}

/// GraphQL variables wrapper for the upsert mutation.
#[derive(Serialize)]
struct GraphqlVariables {
    /// Generated manifest artifact input.
    input: UpsertGeneratedManifestInput,
}

/// Parsed top-level GraphQL response body.
#[derive(Deserialize)]
struct GraphqlResponseBody {
    /// Successful GraphQL response payload, when present.
    data: Option<GraphqlResponseData>,
    #[serde(default)]
    /// GraphQL execution errors, if any.
    errors: Vec<GraphqlError>,
}

/// Parsed `data` object for the upsert mutation response.
#[derive(Deserialize)]
struct GraphqlResponseData {
    #[serde(rename = "upsertGeneratedManifest")]
    /// Upsert mutation payload returned by the API.
    upsert_generated_manifest: Option<UpsertGeneratedManifestPayload>,
}

/// Parsed GraphQL error object.
#[derive(Deserialize)]
struct GraphqlError {
    /// Human-readable GraphQL error message.
    message: String,
}

/// Persist one generated manifest artifact through the GraphQL HTTP endpoint.
///
/// # Errors
///
/// Returns an error if the HTTP request fails or the GraphQL response is invalid.
pub(crate) async fn execute_upsert_generated_manifest_mutation(
    client: &reqwest::Client,
    graphql_url: &str,
    bearer_token: Option<&str>,
    clerk_org_id: Option<&str>,
    input: UpsertGeneratedManifestInput,
) -> Result<String, String> {
    let mut request = client
        .post(graphql_url)
        .header(CONTENT_TYPE, "application/json")
        .json(&GraphqlRequestBody {
            query: UPSERT_GENERATED_MANIFEST_MUTATION,
            variables: GraphqlVariables { input },
        });
    if let Some(bearer_token) = bearer_token {
        request = request.header(AUTHORIZATION, format!("Bearer {bearer_token}"));
    }
    if let Some(clerk_org_id) = clerk_org_id {
        request = request.header(SCRYR_CLERK_ORG_ID_HEADER, clerk_org_id);
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("Failed to call GraphQL endpoint {graphql_url}: {error}"))?;

    let status = response.status();
    let body = response.text().await.map_err(|error| {
        format!("Failed to read GraphQL response body from {graphql_url}: {error}")
    })?;

    if !status.is_success() {
        return Err(format!(
            "GraphQL endpoint {graphql_url} returned HTTP {status}: {body}"
        ));
    }

    let parsed: GraphqlResponseBody = serde_json::from_str(&body).map_err(|error| {
        format!("Failed to parse GraphQL response from {graphql_url}: {error}; body: {body}")
    })?;

    if !parsed.errors.is_empty() {
        let joined_errors = parsed
            .errors
            .into_iter()
            .map(|error| error.message)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!(
            "GraphQL upsertGeneratedManifest mutation failed: {joined_errors}"
        ));
    }

    parsed
        .data
        .and_then(|data| data.upsert_generated_manifest)
        .map(|payload| payload.id.to_string())
        .ok_or_else(|| "GraphQL mutation response did not include an id".to_string())
}
