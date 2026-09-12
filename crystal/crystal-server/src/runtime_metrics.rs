//! On-demand, tenant-scoped Prometheus queries. Ordinary block reads never call this module.
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Source {
    kind: String,
    #[serde(default, alias = "project_id")]
    project_id: Option<u64>,
    #[serde(default)]
    labels: BTreeMap<String, String>,
    #[serde(alias = "query_endpoint")]
    query_endpoint: Option<String>,
    credentials: CredentialRef,
    #[serde(alias = "dashboard_url")]
    dashboard_url: Option<String>,
    environment: String,
    refresh: String,
    window: u32,
    step: u32,
    #[serde(alias = "cache_ttl")]
    cache_ttl: u32,
    #[serde(alias = "ingestion_delay")]
    ingestion_delay: u32,
    queries: BTreeMap<String, String>,
    #[serde(default)]
    units: BTreeMap<String, String>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CredentialRef {
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Connection {
    endpoint: String,
    #[serde(default)]
    username: String,
    #[serde(default, alias = "projectId")]
    project_id: Option<u64>,
    token: String,
}
#[derive(Default)]
struct Entry {
    attempted: Option<Instant>,
    result: Value,
}
type Slot = Arc<Mutex<Entry>>;

pub(crate) struct RuntimeMetrics {
    local: Option<crate::editor::LocalWorkspace>,
    connections: BTreeMap<String, BTreeMap<String, Connection>>,
    cache: Mutex<BTreeMap<String, Slot>>,
    client: reqwest::Client,
}
impl RuntimeMetrics {
    pub(crate) fn from_workspace(
        workspace: Option<&crate::editor::LocalWorkspace>,
    ) -> std::io::Result<Self> {
        use crystal_core::integration_secrets::{Authentication, SecretsFile, secrets_path};
        let entrypoint =
            workspace.map_or_else(|| std::path::Path::new("index.scry"), |w| w.file.as_path());
        let path = secrets_path(entrypoint);
        if std::env::var_os("SCRYR_METRICS_CONNECTIONS_FILE").is_some() {
            return Err(std::io::Error::other(
                "Migrate JSON connections to scryr.secrets.toml and use SCRYR_SECRETS_FILE",
            ));
        }
        let secrets = if path.exists() || std::env::var_os("SCRYR_SECRETS_FILE").is_some() {
            SecretsFile::read(&path).map_err(std::io::Error::other)?
        } else {
            SecretsFile::default()
        };
        let mut connections: BTreeMap<String, BTreeMap<String, Connection>> = BTreeMap::new();
        let mut orgs: Vec<_> = secrets.organizations.keys().cloned().collect();
        orgs.push("local-dev-org".into());
        for org in orgs {
            let scope = secrets.scope(&org);
            let approvals = scope.connections;
            for (name, approval) in approvals {
                let Some(auth) = scope.authentication.get(&name) else {
                    continue;
                };
                let (username, token) = match auth {
                    Authentication::Grafana(s) => (s.username.clone(), s.token.clone()),
                    Authentication::PostHog(s) => (String::new(), s.api_key.clone()),
                    Authentication::GitHub(_) => continue,
                };
                connections.entry(org.clone()).or_default().insert(
                    name,
                    Connection {
                        endpoint: approval.endpoint,
                        project_id: approval.project_id,
                        username,
                        token,
                    },
                );
            }
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(8))
            .build()
            .map_err(std::io::Error::other)?;
        Ok(Self {
            local: workspace.cloned(),
            connections,
            cache: Mutex::new(BTreeMap::new()),
            client,
        })
    }
    /// Execute one local declaration using only this organization's approved connections.
    pub(crate) async fn query(
        &self,
        org: &str,
        config: &Value,
        name: &str,
    ) -> Result<Value, String> {
        if config.to_string().len() > 65536 {
            return Err("Query source exceeds size limit".into());
        }
        let source: Source =
            serde_json::from_value(config.clone()).map_err(|_| "Invalid query source")?;
        source.validate().map_err(str::to_owned)?;
        if !source.queries.contains_key(name) {
            return Err("Unknown query name".into());
        }
        let mut selected = config.clone();
        for field in ["queries", "units", "labels"] {
            if let Some(values) = selected.get_mut(field).and_then(Value::as_object_mut) {
                values.retain(|key, _| key == name);
            }
        }
        let result = self.snapshot(org, "cli", &selected).await;
        if matches!(
            result["status"].as_str(),
            Some("error" | "unavailable" | "stale" | "partial")
        ) {
            return Err(result["error"]
                .as_str()
                .unwrap_or("Query unavailable")
                .to_owned());
        }
        Ok(result)
    }
    pub(crate) async fn load(&self, org: &str, manifests: &Value) -> Value {
        let mut result = serde_json::Map::new();
        if let Some(manifests) = manifests.as_array() {
            let futures = manifests.iter().filter_map(|manifest| {
                let id = manifest.get("manifestId")?.as_str()?;
                let metrics = manifest
                    .get("metrics")
                    .and_then(|m| m.get("provider"))
                    .filter(|v| !v.is_null());
                let analytics = manifest.get("analytics").filter(|v| !v.is_null());
                if metrics.is_none() && analytics.is_none() {
                    return None;
                }
                Some(async move {
                    let (mut snapshot, analytics) = tokio::join!(
                        async {
                            if let Some(config) = metrics {
                                self.snapshot(org, id, config).await
                            } else {
                                json!({})
                            }
                        },
                        async {
                            if let Some(config) = analytics {
                                Some(self.snapshot(org, id, config).await)
                            } else {
                                None
                            }
                        }
                    );
                    if let Some(analytics) = analytics {
                        snapshot["analytics"] = analytics;
                    }
                    (id.to_owned(), snapshot)
                })
            });
            for (id, snapshot) in futures_util::future::join_all(futures).await {
                result.insert(id, snapshot);
            }
        }
        if let Some(manifests) = manifests.as_array() {
            let pending = manifests.iter().filter_map(|manifest| {
                let id = manifest["manifestId"].as_str()?;
                let cards = manifest["cards"].as_array()?;
                Some(async move {
                    let pending =
                        cards.iter().filter_map(|card| {
                            let card_id = card["id"].as_str()?;
                            let source = card.get("source").filter(|s| !s.is_null())?;
                            Some(async move {
                                (card_id.to_owned(), self.snapshot(org, id, source).await)
                            })
                        });
                    let cards: serde_json::Map<_, _> = futures_util::future::join_all(pending)
                        .await
                        .into_iter()
                        .collect();
                    (id.to_owned(), Value::Object(cards))
                })
            });
            for (id, cards) in futures_util::future::join_all(pending).await {
                result.entry(id).or_insert_with(|| json!({}))["cards"] = cards;
            }
        }
        Value::Object(result)
    }
    async fn snapshot(&self, org: &str, id: &str, config: &Value) -> Value {
        let Ok(source) = serde_json::from_value::<Source>(config.clone()) else {
            return unavailable("Invalid metric source configuration");
        };
        if let Err(message) = source.validate() {
            return unavailable(message);
        }
        let key = json!([org, id, config]).to_string();
        let slot = {
            let mut cache = self.cache.lock().await;
            // Bound memory; evict only idle, expired slots, never in-flight queries.
            cache.retain(|_, slot| {
                slot.try_lock().map_or(true, |entry| {
                    entry
                        .attempted
                        .is_none_or(|t| t.elapsed() < Duration::from_hours(2))
                })
            });
            if cache.len() >= 256 && !cache.contains_key(&key) {
                return unavailable("Metrics cache capacity reached");
            }
            Arc::clone(cache.entry(key).or_default())
        };
        let mut entry = slot.lock().await;
        if entry
            .attempted
            .is_some_and(|t| t.elapsed() < Duration::from_secs(u64::from(source.cache_ttl)))
        {
            return entry.result.clone();
        }
        let collected = self.collect(org, &source).await;
        entry.attempted = Some(Instant::now());
        entry.result = match collected {
            Ok(value) => value,
            Err(message) => {
                if entry.result["values"]
                    .as_object()
                    .is_some_and(|v| !v.is_empty())
                {
                    let mut prior = entry.result.clone();
                    prior["status"] = json!("stale");
                    prior["error"] = json!(message);
                    prior
                } else {
                    unavailable(message)
                }
            }
        };
        entry.result.clone()
    }
    #[allow(clippy::literal_string_with_formatting_args)] // These are documented HogQL template tokens.
    async fn collect(&self, org: &str, source: &Source) -> Result<Value, &'static str> {
        let local_connection = if org == "local-dev-org" {
            self.local
                .as_ref()
                .map(|local| local_connection(local, &source.credentials.name))
                .transpose()?
        } else {
            None
        };
        let connection = local_connection
            .as_ref()
            .or_else(|| {
                self.connections
                    .get(org)
                    .and_then(|connections| connections.get(&source.credentials.name))
            })
            .ok_or("Metric connection is not configured for this organization")?;
        let endpoint = approved_endpoint(connection, source)?;
        let end = chrono::Utc::now().timestamp() - i64::from(source.ingestion_delay);
        let start = end - i64::from(source.window);
        let futures = source.queries.iter().map(|(name, expression)| async {
            let request = if source.kind == "posthog" {
                let environment = format!("'{}'", source.environment.replace('\\', "\\\\").replace('\'', "\\'"));
                let query = expression.replace("{start}", &start.to_string()).replace("{end}", &end.to_string()).replace("{environment}", &environment);
                self.client.post(endpoint.clone()).bearer_auth(&connection.token)
                    .json(&json!({"query":{"kind":"HogQLQuery","query":query},"refresh":"force_blocking","name":"Scryr diagram analytics"}))
            } else {
                self.client.get(endpoint.clone()).basic_auth(&connection.username, Some(&connection.token))
                    .query(&[("query", expression.clone()), ("start", start.to_string()), ("end", end.to_string()), ("step", source.step.to_string()), ("timeout", "5s".into())])
            };
            let mut response = request.send().await.map_err(|_| "Metrics request failed or timed out")?;
            if !response.status().is_success() {
                return Err("Metrics backend rejected the request");
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "Cannot read metric response")?
            {
                if bytes.len() + chunk.len() > 1_000_000 {
                    return Err("Metric response exceeds size limit");
                }
                bytes.extend_from_slice(&chunk);
            }
            let body: Value =
                serde_json::from_slice(&bytes).map_err(|_| "Invalid metric response")?;
            let series = if source.kind == "posthog" { parse_aggregate(&body, end)? } else { parse_series(&body, start, end, source.step)? };
            Ok((name.clone(), series))
        });
        let mut values = serde_json::Map::new();
        let mut missing = Vec::new();
        for (name, series) in futures_util::future::try_join_all(futures).await? {
            if let Some(last) = series.last() {
                values.insert(name.clone(),json!({"value":last.1,"evaluatedAt":last.0,"unit":source.units.get(&name),"samples":series,"label":source.labels.get(&name)}));
            } else {
                missing.push(name);
            }
        }
        let status = if values.is_empty() {
            "no_data"
        } else if !missing.is_empty() {
            "partial"
        } else {
            "ready"
        };
        let dashboard = source
            .dashboard_url
            .as_ref()
            .and_then(|s| url::Url::parse(s).ok())
            .filter(|u| u.scheme() == "https")
            .map(|mut u| {
                if source.kind == "prometheus" {
                    u.query_pairs_mut()
                        .append_pair("from", &(start * 1000).to_string())
                        .append_pair("to", &(end * 1000).to_string());
                }
                u.to_string()
            });
        Ok(
            json!({"status":status,"values":values,"missing":missing,"fetchedAt":chrono::Utc::now(),"windowStart":start,"windowEnd":end,"environment":source.environment,"dashboardUrl":dashboard,"source":source.kind}),
        )
    }
}
/// Resolve credentials against the latest *natively checked* local source, never browser input.
fn local_connection(
    local: &crate::editor::LocalWorkspace,
    name: &str,
) -> Result<Connection, &'static str> {
    use crystal_core::integration_secrets::{
        Authentication, ConnectionApproval, SecretsFile, secrets_path,
    };
    let secrets = SecretsFile::read(&secrets_path(&local.file))
        .map_err(|_| "Local TOML credentials are unavailable or invalid")?;
    let auth = secrets
        .authentication
        .get(name)
        .ok_or("Local authentication is not configured")?;
    let envelope = local
        .integrations
        .read()
        .map_err(|_| "Local integration configuration unavailable")?;
    let mut selected: Option<ConnectionApproval> = None;
    for manifest in envelope["manifests"].as_array().into_iter().flatten() {
        let mut sources = vec![
            manifest["metrics"]["provider"].clone(),
            manifest["analytics"].clone(),
        ];
        for integration in manifest["integrations"].as_array().into_iter().flatten() {
            sources.push(
                json!({"credentials":{"name":integration["authentication"]["id"]},
                "queryEndpoint":integration["endpoint"], "projectId":integration["projectId"]}),
            );
        }
        for source in sources {
            if source["credentials"]["name"].as_str() != Some(name) {
                continue;
            }
            // Legacy provider models serialize Python field names; typed integrations
            // above use the public JSON aliases. Accept both checked representations.
            let Some(endpoint) = source
                .get("queryEndpoint")
                .or_else(|| source.get("query_endpoint"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let approval = ConnectionApproval {
                endpoint: endpoint.into(),
                project_id: source
                    .get("projectId")
                    .or_else(|| source.get("project_id"))
                    .and_then(Value::as_u64),
            };
            if let Some(previous) = &selected
                && (previous.endpoint != approval.endpoint
                    || previous.project_id != approval.project_id)
            {
                return Err(
                    "Use separate authentication declarations for different integration destinations",
                );
            }
            selected = Some(approval);
        }
    }
    drop(envelope);
    let approval =
        selected.ok_or("Integration destination is not approved by the checked local source")?;
    let (username, token) = match auth {
        Authentication::Grafana(s) => (s.username.clone(), s.token.clone()),
        Authentication::PostHog(s) => (String::new(), s.api_key.clone()),
        Authentication::GitHub(_) => return Err("Incompatible metric authentication"),
    };
    Ok(Connection {
        endpoint: approval.endpoint,
        project_id: approval.project_id,
        username,
        token,
    })
}

fn unavailable(message: &str) -> Value {
    json!({"status":"error","error":message,"values":{}})
}
impl Source {
    fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.kind.as_str(), "prometheus" | "posthog")
            || self.project_id == Some(0)
            || self
                .labels
                .iter()
                .any(|(k, v)| !self.queries.contains_key(k) || v.trim().is_empty() || v.len() > 80)
            || (self.kind == "posthog"
                && self.queries.values().any(|q| {
                    !["{start}", "{end}", "{environment}"]
                        .iter()
                        .all(|token| q.contains(token))
                }))
            || self.refresh != "on_diagram_load"
            || !(60..=86400).contains(&self.window)
            || !(15..=3600).contains(&self.step)
            || self.window / self.step > 1440
            || !(1..=3600).contains(&self.cache_ttl)
            || self.ingestion_delay > 3600
            || self.queries.is_empty()
            || self.queries.len() > 12
            || self.credentials.name.is_empty()
            || self.credentials.name.len() > 128
            || self.environment.len() > 128
            || self
                .queries
                .iter()
                .any(|(k, v)| k.is_empty() || k.len() > 64 || v.trim().is_empty() || v.len() > 4096)
        {
            return Err("Invalid metric source configuration");
        }
        Ok(())
    }
}
fn approved_endpoint(connection: &Connection, source: &Source) -> Result<url::Url, &'static str> {
    let mut url =
        url::Url::parse(&connection.endpoint).map_err(|_| "Invalid server metric endpoint")?;
    // HTTP is only supported for explicitly configured loopback test backends.
    let loopback = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
    if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Invalid server metric endpoint");
    }
    if let Some(endpoint) = &source.query_endpoint
        && url::Url::parse(endpoint).ok().as_ref() != Some(&url)
    {
        return Err("Manifest metric endpoint is not approved for this connection");
    }
    let path = if source.kind == "posthog" {
        let project = connection
            .project_id
            .filter(|id| *id > 0)
            .ok_or("PostHog project is not configured for this connection")?;
        if source.project_id.is_some_and(|id| id != project) {
            return Err("PostHog project is not approved for this connection");
        }
        format!(
            "{}/api/projects/{project}/query/",
            url.path().trim_end_matches('/')
        )
    } else {
        format!("{}/api/v1/query_range", url.path().trim_end_matches('/'))
    };
    url.set_path(&path);
    Ok(url)
}
fn parse_aggregate(body: &Value, end: i64) -> Result<Vec<(f64, f64)>, &'static str> {
    if body.get("error").is_some_and(|v| !v.is_null())
        || body["is_cached"] == true
        || body["query_status"]["complete"] == false
        || body["hasMore"] == true
    {
        return Err("PostHog returned an incomplete or cached result");
    }
    let rows = body["results"]
        .as_array()
        .ok_or("Missing PostHog results")?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    if rows.len() != 1 || rows[0].as_array().is_none_or(|r| r.len() != 1) {
        return Err("PostHog query must return one numeric cell");
    }
    if rows[0][0].is_null() {
        return Ok(Vec::new());
    }
    let value = rows[0][0]
        .as_f64()
        .filter(|v| v.is_finite())
        .ok_or("Invalid PostHog aggregate")?;
    #[allow(clippy::cast_precision_loss)]
    Ok(vec![(end as f64, value)])
}
fn parse_series(
    body: &Value,
    start: i64,
    end: i64,
    step: u32,
) -> Result<Vec<(f64, f64)>, &'static str> {
    if body["status"] != "success"
        || body
            .get("warnings")
            .and_then(Value::as_array)
            .is_some_and(|v| !v.is_empty())
        || body["data"]["resultType"] != "matrix"
    {
        return Err("Metrics backend returned an incomplete or invalid result");
    }
    let results = body["data"]["result"]
        .as_array()
        .ok_or("Missing metric results")?;
    if results.len() > 1 {
        return Err("Metric query must aggregate to a single series");
    }
    let Some(series) = results.first() else {
        return Ok(Vec::new());
    };
    let samples = series["values"]
        .as_array()
        .ok_or("Missing metric samples")?;
    if samples.len() > 1441 {
        return Err("Too many metric samples");
    }
    let mut output = Vec::new();
    for point in samples {
        let timestamp = point[0].as_f64().ok_or("Invalid metric timestamp")?;
        let value = point[1]
            .as_str()
            .ok_or("Invalid metric value")?
            .parse::<f64>()
            .map_err(|_| "Invalid metric number")?;
        if !timestamp.is_finite() {
            return Err("Invalid metric timestamp");
        }
        #[allow(clippy::cast_precision_loss)]
        if timestamp < (start as f64) || timestamp > (end as f64) {
            return Err("Metric timestamp outside requested window");
        }
        if value.is_finite() {
            output.push((timestamp, value));
        }
    }
    output.sort_by(|a, b| a.0.total_cmp(&b.0));
    #[allow(clippy::cast_precision_loss)]
    if output
        .last()
        .is_some_and(|last| (end as f64) - last.0 > f64::from(step) * 2.0)
    {
        return Ok(Vec::new());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn local_credentials_resolve_checked_legacy_and_typed_destinations()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let file = folder.path().join("index.scry");
        std::fs::write(&file, "")?;
        std::fs::write(
            folder.path().join("scryr.secrets.toml"),
            "[authentication.grafana]\nusername = '123'\ntoken = 'metrics-read'\n\
             [authentication.posthog]\napi_key = 'query-read'\n",
        )?;
        let workspace = crate::editor::LocalWorkspace::new(
            folder.path(),
            &file,
            Arc::new(|_, _| Err("Unused validator".into())),
        )?;
        assert!(local_connection(&workspace, "grafana").is_err());
        // These snake_case fields are emitted by legacy PrometheusSource/PostHogSource.
        let legacy = json!({"manifests":[{
            "metrics":{"provider":{"credentials":{"name":"grafana"},
                "query_endpoint":"https://metrics.example/api/prom"}},
            "analytics":{"credentials":{"name":"posthog"},
                "query_endpoint":"https://us.posthog.com", "project_id":598_963}
        }]});
        let aliases = json!({"manifests":[{
            "metrics":{"provider":{"credentials":{"name":"grafana"},
                "queryEndpoint":"https://metrics.example/api/prom"}},
            "analytics":{"credentials":{"name":"posthog"},
                "queryEndpoint":"https://us.posthog.com", "projectId":598_963}
        }]});
        let typed = json!({"manifests":[{"integrations":[
            {"authentication":{"id":"grafana"},"endpoint":"https://metrics.example/api/prom"},
            {"authentication":{"id":"posthog"},"endpoint":"https://us.posthog.com","projectId":598_963}
        ]}]});
        for envelope in [legacy.clone(), aliases, typed] {
            workspace.set_integrations(envelope)?;
            let grafana = local_connection(&workspace, "grafana")?;
            assert_eq!(grafana.endpoint, "https://metrics.example/api/prom");
            assert_eq!(grafana.username, "123");
            assert_eq!(grafana.token, "metrics-read");
            let posthog = local_connection(&workspace, "posthog")?;
            assert_eq!(posthog.endpoint, "https://us.posthog.com");
            assert_eq!(posthog.project_id, Some(598_963));
            assert_eq!(posthog.token, "query-read");
            assert!(local_connection(&workspace, "undeclared").is_err());
        }
        let mut conflicting = legacy;
        conflicting["manifests"][0]["integrations"] = json!([
            {"authentication":{"id":"posthog"},"endpoint":"https://us.posthog.com","projectId":42}
        ]);
        workspace.set_integrations(conflicting)?;
        assert!(local_connection(&workspace, "posthog").is_err());
        Ok(())
    }

    async fn expire(runtime: &RuntimeMetrics) {
        for slot in runtime.cache.lock().await.values() {
            slot.lock().await.attempted = Instant::now().checked_sub(Duration::from_secs(2));
        }
    }
    #[actix_web::test]
    async fn load_deduplicates_caches_isolates_and_preserves_failures()
    -> Result<(), Box<dyn std::error::Error>> {
        let calls = Arc::new(AtomicUsize::new(0));
        let mode = Arc::new(AtomicUsize::new(0));
        let observed_calls = Arc::clone(&calls);
        let observed_mode = Arc::clone(&mode);
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let server=actix_web::HttpServer::new(move || {
            let calls=Arc::clone(&observed_calls);let mode=Arc::clone(&observed_mode);
            actix_web::App::new().route("/api/v1/query_range",actix_web::web::get().to(move |request:actix_web::HttpRequest| {
                let calls=Arc::clone(&calls);let mode=Arc::clone(&mode);
                async move {
                    calls.fetch_add(1,Ordering::SeqCst);
                    assert_eq!(request.headers().get("authorization").and_then(|s|s.to_str().ok()),Some("Basic dXNlcjpzZWNyZXQ="));
                    if mode.load(Ordering::SeqCst)==1 {return actix_web::HttpResponse::InternalServerError().finish();}
                    let end=url::form_urlencoded::parse(request.query_string().as_bytes()).find(|(key,_)|key=="end").and_then(|(_,v)|v.parse::<i64>().ok()).unwrap_or_default();
                    let results=if mode.load(Ordering::SeqCst)==2 {json!([])}else{json!([{"values":[[end,"12.5"]]}])};
                    actix_web::HttpResponse::Ok().json(json!({"status":"success","data":{"resultType":"matrix","result":results}}))
                }
            }))
        }).listen(listener)?.run();
        let handle = server.handle();
        actix_web::rt::spawn(server);
        let runtime = RuntimeMetrics {
            local: None,
            connections: BTreeMap::from([(
                "org".into(),
                BTreeMap::from([(
                    "grafana".into(),
                    Connection {
                        endpoint: format!("http://{address}"),
                        username: "user".into(),
                        project_id: None,
                        token: "secret".into(),
                    },
                )]),
            )]),
            cache: Mutex::default(),
            client: reqwest::Client::new(),
        };
        let source = json!({"kind":"prometheus","credentials":{"name":"grafana"},"environment":"production","refresh":"on_diagram_load","window":900,"step":60,"cacheTtl":1,"ingestionDelay":120,"queries":{"requestRate":"sum(rate(requests[5m]))"}});
        let manifests = json!([{"manifestId":"api","metrics":{"provider":source}}]);
        let (a, b) = tokio::join!(
            runtime.load("org", &manifests),
            runtime.load("org", &manifests)
        );
        assert_eq!(a, b);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(a["api"]["values"]["requestRate"]["value"], 12.5);
        runtime.load("org", &manifests).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        // Two views sharing an identical query retain separate card IDs and one cached fetch.
        let typed = json!([{"manifestId":"api","cards":[
            {"id":"grafana_performance","source":source},
            {"id":"grafana_uptime","source":source},
            {"id":"posthog_performance","source":null}
        ]}]);
        let typed_result = runtime.load("org", &typed).await;
        assert_eq!(
            typed_result["api"]["cards"]["grafana_performance"]["values"]["requestRate"]["value"],
            12.5
        );
        assert_eq!(
            typed_result["api"]["cards"]["grafana_uptime"]["status"],
            "ready"
        );
        assert!(
            typed_result["api"]["cards"]
                .get("posthog_performance")
                .is_none()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let denied_cards = runtime.load("other", &typed).await;
        assert_eq!(
            denied_cards["api"]["cards"]["grafana_uptime"]["status"],
            "error"
        );
        let denied = runtime.load("other", &manifests).await;
        assert_eq!(denied["api"]["status"], "error");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let mut changed = manifests.clone();
        changed[0]["metrics"]["provider"]["queryEndpoint"] = json!("https://unapproved.example");
        assert_eq!(
            runtime.load("org", &changed).await["api"]["status"],
            "error"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        expire(&runtime).await;
        mode.store(1, Ordering::SeqCst);
        let failure = runtime.load("org", &manifests).await;
        assert_eq!(failure["api"]["status"], "stale");
        assert_eq!(failure["api"]["values"], a["api"]["values"]);
        assert!(!failure.to_string().contains("secret"));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        runtime.load("org", &manifests).await;
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        expire(&runtime).await;
        mode.store(2, Ordering::SeqCst);
        assert_eq!(
            runtime.load("org", &manifests).await["api"]["status"],
            "no_data"
        );
        handle.stop(true).await;
        Ok(())
    }
    #[actix_web::test]
    async fn posthog_aggregates_are_authenticated_cached_and_isolated()
    -> Result<(), Box<dyn std::error::Error>> {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let server = actix_web::HttpServer::new(move || {
            let calls = Arc::clone(&observed);
            actix_web::App::new().route(
                "/api/projects/42/query/",
                actix_web::web::post().to(
                    move |request: actix_web::HttpRequest, body: actix_web::web::Json<Value>| {
                        let calls = Arc::clone(&calls);
                        async move {
                            calls.fetch_add(1, Ordering::SeqCst);
                            assert_eq!(
                                request
                                    .headers()
                                    .get("authorization")
                                    .and_then(|h| h.to_str().ok()),
                                Some("Bearer secret")
                            );
                            assert_eq!(body["refresh"], "force_blocking");
                            assert_eq!(body["query"]["kind"], "HogQLQuery");
                            let query = body["query"]["query"].as_str().unwrap_or_default();
                            assert!(query.contains("'production'"));
                            assert!(!query.contains("{start}"));
                            actix_web::HttpResponse::Ok()
                                .json(json!({"results":[[0]],"is_cached":false}))
                        }
                    },
                ),
            )
        })
        .listen(listener)?
        .run();
        let handle = server.handle();
        actix_web::rt::spawn(server);
        let runtime = RuntimeMetrics {
            local: None,
            connections: BTreeMap::from([(
                "org".into(),
                BTreeMap::from([(
                    "posthog".into(),
                    Connection {
                        endpoint: format!("http://{address}"),
                        username: String::new(),
                        project_id: Some(42),
                        token: "secret".into(),
                    },
                )]),
            )]),
            cache: Mutex::default(),
            client: reqwest::Client::new(),
        };
        let source = json!({"kind":"posthog","credentials":{"name":"posthog"},"projectId":42,"environment":"production","refresh":"on_diagram_load","window":86400,"step":60,"cacheTtl":60,"ingestionDelay":120,"queries":{"views":"SELECT count() FROM events WHERE timestamp >= toDateTime({start}) AND timestamp < toDateTime({end}) AND properties.environment = {environment}"},"labels":{"views":"Catalog views"}});
        let manifests = json!([{"manifestId":"web","analytics":source, "metrics":{"provider":{"invalid":true}}}]);
        let (a, b) = tokio::join!(
            runtime.load("org", &manifests),
            runtime.load("org", &manifests)
        );
        assert_eq!(a, b);
        assert_eq!(a["web"]["status"], "error");
        assert_eq!(a["web"]["analytics"]["status"], "ready");
        assert_eq!(a["web"]["analytics"]["values"]["views"]["value"], 0.0);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            runtime.load("other", &manifests).await["web"]["analytics"]["status"],
            "error"
        );
        let mut denied = manifests.clone();
        denied[0]["analytics"]["projectId"] = json!(99);
        assert_eq!(
            runtime.load("org", &denied).await["web"]["analytics"]["status"],
            "error"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let selected = runtime.query("org", &source, "views").await?;
        assert_eq!(selected["values"]["views"]["value"], 0.0);
        assert!(runtime.query("other", &source, "views").await.is_err());
        assert!(runtime.query("org", &source, "unknown").await.is_err());
        handle.stop(true).await;
        Ok(())
    }
    #[test]
    fn posthog_rejects_partial_cached_and_nonaggregate_results() {
        assert!(parse_aggregate(&json!({"results":[]}), 100).is_ok_and(|v| v.is_empty()));
        for body in [
            json!({"results":[[1],[2]]}),
            json!({"results":[["1"]]}),
            json!({"results":[[1]],"is_cached":true}),
            json!({"results":[[1]],"query_status":{"complete":false}}),
        ] {
            assert!(parse_aggregate(&body, 100).is_err());
        }
    }
    #[test]
    fn rejects_ambiguous_series_and_ignores_nonfinite_samples() -> Result<(), &'static str> {
        let body = json!({"status":"success","data":{"resultType":"matrix","result":[{"values":[[100,"NaN"],[160,"+Inf"]]}]}});
        assert!(parse_series(&body, 100, 160, 60)?.is_empty());
        let mut multiple = body;
        multiple["data"]["result"] = json!([{"values":[]},{"values":[]}]);
        assert!(parse_series(&multiple, 100, 160, 60).is_err());
        let old = json!({"status":"success","data":{"resultType":"matrix","result":[{"values":[[100,"2"]]}]}});
        assert!(parse_series(&old, 100, 300, 60)?.is_empty());
        Ok(())
    }
}
