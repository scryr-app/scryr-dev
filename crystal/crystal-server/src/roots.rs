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
        crate::samples::ensure_samples(state, context)
            .await
            .map_err(async_graphql::Error::new)?;
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
    /// Read passive, immutable collector history scoped to the active organization.
    #[allow(clippy::too_many_arguments)] // GraphQL exposes each typed scope selector explicitly.
    async fn evidence_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        manifest_id: String,
        section: crystal_core::evidence::EvidenceSection,
        collector_id: String,
        workspace_id: String,
        #[graphql(default = 50)] limit: u32,
        #[graphql(default = 0)] offset: u32,
    ) -> async_graphql::Result<Vec<crystal_core::evidence::EvidenceObservation>> {
        let context = ctx.data::<crystal_core::manifest::ManifestRequestContext>()?;
        let pool = ctx.data::<persistence::DatabasePool>()?;
        persistence::evidence_history(
            pool,
            &context.clerk_org_id,
            &manifest_id,
            section,
            &collector_id,
            &workspace_id,
            limit,
            offset,
        )
        .await
        .map_err(async_graphql::Error::new)
    }

    /// Immutable, organization-scoped observation used for on-demand detail pagination.
    async fn evidence_observation(
        &self,
        ctx: &async_graphql::Context<'_>,
        observation_id: String,
        workspace_id: Option<String>,
    ) -> async_graphql::Result<Option<crystal_core::evidence::EvidenceObservation>> {
        let context = ctx.data::<crystal_core::manifest::ManifestRequestContext>()?;
        let pool = ctx.data::<persistence::DatabasePool>()?;
        persistence::evidence_observation(
            pool,
            &context.clerk_org_id,
            &observation_id,
            workspace_id.as_deref(),
        )
        .await
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

        crate::samples::ensure_samples(state, &request_context).await?;

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
        workspace_id: Option<String>,
    ) -> Result<Vec<Block>, String> {
        let auth = require_authenticated_request(ctx).map_err(|error| error.message)?;
        let request_context = auth.manifest_request_context()?;
        let state = ctx.data::<AppState>().map_err(|e| e.message)?;
        let pool = &state.db_pool;
        crate::samples::ensure_samples(state, &request_context).await?;
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

        let declarations = crystal_core::collectors::declarations_for_projection(&raw_json)?;
        let evidence = persistence::project_evidence(
            pool,
            &request_context.clerk_org_id,
            &declarations,
            workspace_id.as_deref(),
        )
        .await?;
        let mut blocks = Vec::new();
        if let Some(items) = raw_json.as_array() {
            for item in items {
                let id = item.get("manifestId").and_then(serde_json::Value::as_str);
                let collector_evidence = evidence
                    .iter()
                    .filter(|e| Some(e.manifest_id.as_str()) == id)
                    .cloned()
                    .collect();
                blocks.push(Block {
                    raw_json: item.clone(),
                    collector_evidence,
                });
            }
        }

        Ok(blocks)
    }
}

/// Root mutation type for GraphQL schema.
pub(crate) type MutationRoot = GeneratedManifestMutationRoot;

/// Root subscription type for GraphQL schema.
pub(crate) type SubscriptionRoot = async_graphql::EmptySubscription;
