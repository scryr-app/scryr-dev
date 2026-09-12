//! Execute named declarations through the server's configured provider connections.
use super::workflow::Project;
use crate::args::QueryArgs;
use serde_json::{Value, json};

/// List or execute a selected declaration without publishing the manifest.
pub(crate) async fn run(args: QueryArgs) -> Result<(), String> {
    let project = Project::new(args.common)?;
    let envelope: Value = serde_json::from_str(&project.json()?).map_err(|e| e.to_string())?;
    let manifests = envelope["manifests"]
        .as_array()
        .ok_or("Missing manifests")?;
    let mut choices = Vec::new();
    for manifest in manifests {
        if args
            .manifest
            .as_deref()
            .is_some_and(|s| !super::report_config::matches(manifest, s))
        {
            continue;
        }
        let mut sources = Vec::new();
        if args.card.is_none() {
            sources.extend([
                (&manifest["metrics"]["provider"], None),
                (&manifest["analytics"], None),
            ]);
        }
        for card in manifest["cards"].as_array().into_iter().flatten() {
            if args
                .card
                .as_deref()
                .is_none_or(|id| card["id"].as_str() == Some(id))
            {
                sources.push((&card["source"], card["id"].as_str()));
            }
        }
        for (config, card_id) in sources {
            if args
                .provider
                .as_deref()
                .is_some_and(|p| config["kind"].as_str() != Some(p))
            {
                continue;
            }
            if let Some(queries) = config["queries"].as_object() {
                for name in queries.keys() {
                    choices.push((manifest, config, name, card_id));
                }
            }
        }
    }
    if args.list {
        let rows: Vec<Value> = choices.iter().map(|(m, c, n, card)| json!({
            "manifest": m["variable_name"], "manifestId":m["manifestId"], "name": n, "provider":c["kind"], "card":card
        })).collect();
        if args.json {
            println!("{}", json!(rows));
        } else {
            for row in rows {
                println!(
                    "{}  {}  {}  {}",
                    row["manifest"], row["card"], row["name"], row["provider"]
                );
            }
        }
        return Ok(());
    }
    let name = args
        .name
        .as_deref()
        .ok_or("Provide a query name or --list")?;
    let selected: Vec<_> = choices
        .into_iter()
        .filter(|(_, _, n, _)| n.as_str() == name)
        .collect();
    let (_, config, _, _) = match selected.as_slice() {
        [choice] => choice,
        [] => return Err(format!("No query named {name}; use scryr query --list")),
        _ => {
            return Err(format!(
                "Query {name} is ambiguous; select --manifest and --card (or --provider for legacy sources)"
            ));
        }
    };
    execute(&project, config, name, args.json).await
}

/// Execute a selected source through the authenticated server query endpoint.
async fn execute(
    project: &Project,
    config: &Value,
    name: &str,
    json_output: bool,
) -> Result<(), String> {
    let target = project
        .args
        .graphql_url
        .clone()
        .unwrap_or_else(super::generate::upload::default_local_graphql_url);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let mut request = client.post(super::report::endpoint(&target)?).json(&json!({
        "query":"query Run($source: JSON!, $name: String!) { manifestQuery(source: $source, name: $name) }",
        "variables":{"source":config,"name":name}
    }));
    if let Some(token) = super::report::reporting_token(&target).await? {
        request = request.bearer_auth(token);
    }
    if let Some(org) = &project.args.clerk_org_id {
        request = request.header("X-Scryr-Clerk-Org-Id", org);
    }
    let response = request
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let body: Value = response.json().await.map_err(|e| e.to_string())?;
    if let Some(errors) = body.get("errors") {
        return Err(format!("Query failed: {errors}"));
    }
    let result = body
        .pointer("/data/manifestQuery")
        .filter(|v| !v.is_null())
        .ok_or("Server returned no query result")?;
    if json_output {
        println!("{result}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(result).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}
