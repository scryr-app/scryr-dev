//! GraphQL root resolvers.

use crate::auth::require_authenticated_request;
use crate::generated_manifest_graphql::GeneratedManifestMutationRoot;
use crate::health::run_healthcheck;
use crate::state::AppState;
use async_graphql::Object;
use crystal_core::graphql_types::{Block, HealthStatus, ScryrMap};
use crystal_core::persistence;

/// Attach durable workflow history to the generated block without rewriting its artifact.
async fn attach_action_history(
    pool: &persistence::DatabasePool,
    clerk_org_id: &str,
    raw_json: &mut serde_json::Value,
) -> Result<(), String> {
    let Some(manifest_id) = raw_json
        .get("manifestId")
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(());
    };
    let history = persistence::read_action_history(pool, clerk_org_id, manifest_id, 100, 0).await?;
    let Some(last_build) = history.runs.first().map(|run| run.updated_at) else {
        return Ok(());
    };
    let status = history.build_status();
    let cicd = raw_json
        .as_object_mut()
        .ok_or("generated manifest block must be an object")?
        .entry("cicd")
        .or_insert_with(|| serde_json::json!({}));
    let cicd = cicd
        .as_object_mut()
        .ok_or("generated manifest cicd section must be an object")?;
    cicd.insert(
        "githubActions".into(),
        serde_json::to_value(history).map_err(|error| error.to_string())?,
    );
    cicd.insert("lastBuild".into(), serde_json::json!(last_build));
    if let Some(status) = status {
        cicd.insert("buildStatus".into(), serde_json::json!(status));
    } else {
        cicd.remove("buildStatus");
    }
    Ok(())
}

/// Root query type for GraphQL schema.
pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Read the selected diagram's complete source document.
    async fn manifest_document(
        &self,
        ctx: &async_graphql::Context<'_>,
        identifier: String,
    ) -> async_graphql::Result<
        async_graphql::Json<crystal_core::persistence::documents::ManifestDocument>,
    > {
        let context = ctx.data::<crystal_core::manifest::ManifestRequestContext>()?;
        let state = ctx.data::<AppState>()?;
        if state.auth_mode == crate::state::AuthMode::Local
            && !ctx
                .data_opt::<crate::editor::EditorRequestAllowed>()
                .is_some_and(|v| v.0)
        {
            return Err(async_graphql::Error::new(
                "Local editor requests require the server's own origin",
            ));
        }
        let editor = ctx.data::<crate::editor::EditorService>()?;
        editor
            .read(&state.db_pool, context, &identifier)
            .await
            .map(async_graphql::Json)
            .map_err(async_graphql::Error::new)
    }
    /// Execute a named local declaration without persisting a diagram.
    async fn manifest_query(
        &self,
        ctx: &async_graphql::Context<'_>,
        source: async_graphql::Json<serde_json::Value>,
        name: String,
    ) -> Result<async_graphql::Json<serde_json::Value>, String> {
        let auth = require_authenticated_request(ctx).map_err(|error| error.message)?;
        let context = auth.manifest_request_context()?;
        let metrics = ctx
            .data::<crate::runtime_metrics::RuntimeMetrics>()
            .map_err(|e| e.message)?;
        metrics
            .query(&context.clerk_org_id, &source.0, &name)
            .await
            .map(async_graphql::Json)
    }

    /// Fetch configured runtime metrics once when opening a diagram. Block polling never calls this.
    async fn diagram_metrics(
        &self,
        ctx: &async_graphql::Context<'_>,
        scry_identifier: Option<String>,
        sample: Option<String>,
    ) -> Result<async_graphql::Json<serde_json::Value>, String> {
        let auth = require_authenticated_request(ctx).map_err(|error| error.message)?;
        let context = auth.manifest_request_context()?;
        let state = ctx.data::<AppState>().map_err(|e| e.message)?;
        let manifests = if let Some(id) = scry_identifier.as_deref().filter(|s| !s.is_empty()) {
            persistence::read_generated_manifest_json_by_scry_identifier(
                &state.db_pool,
                &context.clerk_org_id,
                id,
            )
            .await?
        } else {
            persistence::read_generated_manifest_json(
                &state.db_pool,
                &context.clerk_org_id,
                sample.as_deref().or(state.sample.as_deref()),
            )
            .await?
        };
        let metrics = ctx
            .data::<crate::runtime_metrics::RuntimeMetrics>()
            .map_err(|e| e.message)?;
        Ok(async_graphql::Json(
            metrics.load(&context.clerk_org_id, &manifests).await,
        ))
    }

    /// Read operational observations for the active organization.
    async fn report_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        manifest_id: String,
        #[graphql(default = 100)] limit: u32,
        #[graphql(default = 0)] offset: u32,
    ) -> async_graphql::Result<async_graphql::Json<Vec<crystal_core::reports::Report>>> {
        let context = ctx.data::<crystal_core::manifest::ManifestRequestContext>()?;
        let pool = ctx.data::<persistence::DatabasePool>()?;
        persistence::read_reports(pool, &context.clerk_org_id, &manifest_id, limit, offset)
            .await
            .map(async_graphql::Json)
            .map_err(async_graphql::Error::new)
    }

    /// Read complete workflow attempts, newest first, scoped to the active organization.
    async fn action_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        manifest_id: String,
        #[graphql(default = 100)] limit: u32,
        #[graphql(default = 0)] offset: u32,
    ) -> async_graphql::Result<async_graphql::Json<crystal_core::action_history::GithubActionsLog>>
    {
        let context = ctx
            .data::<crystal_core::manifest::ManifestRequestContext>()
            .map_err(|_| async_graphql::Error::new("request is missing active organization"))?;
        let pool = ctx.data::<persistence::DatabasePool>()?;
        persistence::read_action_history(pool, &context.clerk_org_id, &manifest_id, limit, offset)
            .await
            .map(async_graphql::Json)
            .map_err(async_graphql::Error::new)
    }

    async fn health(&self, ctx: &async_graphql::Context<'_>) -> HealthStatus {
        let state = match ctx.data::<AppState>() {
            Ok(state) => state,
            Err(error) => {
                let message = error.message;
                return HealthStatus {
                    status: "error".to_string(),
                    database_ok: false,
                    database_message: message,
                };
            }
        };

        run_healthcheck(state).await
    }

    /// Lists persisted Scryr maps available in the configured database.
    async fn scryr_maps(&self, ctx: &async_graphql::Context<'_>) -> Result<Vec<ScryrMap>, String> {
        let auth = require_authenticated_request(ctx).map_err(|error| error.message)?;
        let request_context = auth.manifest_request_context()?;
        let state = ctx.data::<AppState>().map_err(|e| e.message)?;
        let pool = &state.db_pool;

        persistence::list_generated_manifest_maps(pool, &request_context.clerk_org_id)
            .await
            .map_err(String::from)
    }

    /// Loads and returns blocks from generated manifest artifacts in the configured database.
    /// Pass `scry_identifier` to load an exact Scryr map, or `sample` for legacy lookup.
    async fn blocks(
        &self,
        ctx: &async_graphql::Context<'_>,
        scry_identifier: Option<String>,
        sample: Option<String>,
    ) -> Result<Vec<Block>, String> {
        let auth = require_authenticated_request(ctx).map_err(|error| error.message)?;
        let request_context = auth.manifest_request_context()?;
        let state = ctx.data::<AppState>().map_err(|e| e.message)?;
        let pool = &state.db_pool;
        let raw_json = if let Some(scry_identifier) = scry_identifier
            .as_deref()
            .filter(|identifier| !identifier.trim().is_empty())
        {
            persistence::read_generated_manifest_json_by_scry_identifier(
                pool,
                &request_context.clerk_org_id,
                scry_identifier,
            )
            .await?
        } else {
            // Query-level `sample` overrides the server-level default.
            let resolved = sample.as_deref().or(state.sample.as_deref());
            persistence::read_generated_manifest_json(pool, &request_context.clerk_org_id, resolved)
                .await?
        };

        // Operational history is stored separately so reports never rewrite generated artifacts.
        // Enrich the response copy that the map already polls every 30 seconds.
        let mut blocks = Vec::new();
        if let Some(items) = raw_json.as_array() {
            for item in items {
                let mut item = item.clone();
                attach_action_history(pool, &request_context.clerk_org_id, &mut item).await?;
                blocks.push(Block { raw_json: item });
            }
        }

        Ok(blocks)
    }
}

/// Root mutation type for GraphQL schema.
pub(crate) type MutationRoot = GeneratedManifestMutationRoot;

/// Root subscription type for GraphQL schema.
pub(crate) type SubscriptionRoot = async_graphql::EmptySubscription;

#[cfg(test)]
mod tests {
    use super::attach_action_history;
    use crystal_core::{
        action_history::GithubActionRun,
        manifest::ManifestRequestContext,
        persistence::{self, DatabasePool},
    };

    #[tokio::test]
    async fn durable_action_history_enriches_generated_blocks()
    -> Result<(), Box<dyn std::error::Error>> {
        let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let pool = DatabasePool::Sqlite(sqlite.clone());
        let context = ManifestRequestContext {
            clerk_user_id: "user".into(),
            clerk_org_id: "org".into(),
            clerk_org_slug: None,
            clerk_org_role: Some("org:admin".into()),
            clerk_org_permissions: vec![],
        };
        let run: GithubActionRun = serde_json::from_value(serde_json::json!({
            "host": "github.com", "repositoryId": 123, "repository": "example/api",
            "workflowId": 42, "workflowName": "CI", "runId": 12345, "runAttempt": 1,
            "headBranch": "main", "headSha": "abcdef",
            "htmlUrl": "https://github.com/example/api/actions/runs/12345",
            "status": "completed", "conclusion": "success",
            "createdAt": "2026-09-08T10:00:00Z", "updatedAt": "2026-09-08T10:02:00Z"
        }))?;
        persistence::record_action_run(
            &pool,
            &context,
            "services/api",
            run,
            Some("delivery".into()),
            "webhook",
        )
        .await?;

        let mut block = serde_json::json!({
            "manifestId": "services/api",
            "cicd": {"platform": "github_actions"}
        });
        attach_action_history(&pool, "org", &mut block).await?;
        assert_eq!(block["cicd"]["buildStatus"], "passing");
        assert_eq!(block["cicd"]["lastBuild"], "2026-09-08T10:02:00Z");
        assert_eq!(block["cicd"]["githubActions"]["runs"][0]["runId"], 12345);

        sqlite.close().await;
        Ok(())
    }
}
