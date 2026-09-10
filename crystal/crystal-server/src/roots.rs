//! GraphQL root resolvers.

use crate::auth::require_authenticated_request;
use crate::generated_manifest_graphql::GeneratedManifestMutationRoot;
use crate::health::run_healthcheck;
use crate::state::AppState;
use async_graphql::Object;
use crystal_core::graphql_types::{Block, HealthStatus, ScryrMap};
use crystal_core::persistence;

/// Root query type for GraphQL schema.
pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
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

        // Parse the JSON array; each item is already the block object, so keep it intact.
        let blocks = raw_json.as_array().map_or_else(Vec::new, |arr| {
            arr.iter()
                .map(|item| Block {
                    raw_json: item.clone(),
                })
                .collect()
        });

        Ok(blocks)
    }
}

/// Root mutation type for GraphQL schema.
pub(crate) type MutationRoot = GeneratedManifestMutationRoot;

/// Root subscription type for GraphQL schema.
pub(crate) type SubscriptionRoot = async_graphql::EmptySubscription;
