//! Scryr requests shared by provider collection and its scheduler.
use serde_json::{Value, json};
use std::time::Duration;

/// Destination credentials stay in the collector process, never in a manifest.
pub(super) struct Client {
    /// Bounded, redirect-free HTTP transport.
    http: reqwest::Client,
    /// Explicitly selected Scryr destination.
    endpoint: url::Url,
    /// Hosted Scryr credentials, when explicitly targeting a hosted endpoint.
    token: Option<String>,
    /// Active organization for hosted requests.
    org: Option<String>,
}

impl Client {
    /// Resolve authentication once for a collection session.
    pub(super) async fn new(endpoint: &str, org: Option<String>) -> Result<Self, String> {
        Ok(Self {
            http: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| e.to_string())?,
            endpoint: super::super::report::endpoint(endpoint)?,
            token: super::super::report::reporting_token(endpoint).await?,
            org,
        })
    }

    /// Execute a typed operation without logging credentials or raw HTTP bodies.
    pub(super) async fn execute(&self, query: &str, variables: Value) -> Result<Value, String> {
        let mut request = self
            .http
            .post(self.endpoint.clone())
            .json(&json!({"query":query,"variables":variables}));
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        if let Some(org) = &self.org {
            request = request.header("X-Scryr-Clerk-Org-Id", org);
        }
        let body: Value = request
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        if body
            .get("errors")
            .and_then(Value::as_array)
            .is_some_and(|errors| !errors.is_empty())
        {
            return Err("Scryr rejected the provider request; check the endpoint, permissions, and server version".into());
        }
        body.get("data")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or_else(|| "Scryr returned no provider data".into())
    }

    /// Read only maps registered by this serve session; source is not re-executed.
    pub(super) async fn manifests(&self, identifiers: &[String]) -> Result<Vec<Value>, String> {
        let mut manifests = Vec::new();
        for identifier in identifiers {
            let data = self
                .execute(
                    "query($id:String!){blocks(scryIdentifier:$id){rawJsonString}}",
                    json!({"id":identifier}),
                )
                .await?;
            let blocks = data["blocks"]
                .as_array()
                .ok_or("Scryr returned invalid blocks")?;
            for block in blocks {
                manifests.push(
                    serde_json::from_str(
                        block["rawJsonString"]
                            .as_str()
                            .ok_or("Scryr returned invalid block data")?,
                    )
                    .map_err(|e| e.to_string())?,
                );
            }
        }
        Ok(manifests)
    }

    /// Preserve successful timestamps when collection fails.
    pub(super) async fn status(
        &self,
        manifest: &str,
        provider: &str,
        error: Option<&str>,
        context: Option<&Value>,
    ) -> Result<(), String> {
        self.execute("mutation($id:String!,$provider:String!,$error:String,$context:JSON){recordProviderSync(manifestId:$id,provider:$provider,error:$error,context:$context)}",
            json!({"id":manifest,"provider":provider,"error":error,"context":context})).await?;
        Ok(())
    }
}
