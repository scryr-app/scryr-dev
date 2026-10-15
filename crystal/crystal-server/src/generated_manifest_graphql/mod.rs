//! GraphQL mutation support for persisting generated manifest artifacts.

use async_graphql::{Context, Object};
use crystal_core::manifest::ManifestRequestContext;
use crystal_core::manifest::{UpsertGeneratedManifestInput, UpsertGeneratedManifestPayload};
use crystal_core::persistence::{DatabasePool, persist_generated_manifest};

/// Mutation root for generated manifest persistence.
#[derive(Default)]
pub(crate) struct GeneratedManifestMutationRoot;

#[Object]
impl GeneratedManifestMutationRoot {
    /// Save source and all its diagram artifacts with optimistic concurrency.
    async fn save_manifest_document(
        &self,
        ctx: &Context<'_>,
        identifier: String,
        revision: String,
        envelope: async_graphql::Json<serde_json::Value>,
    ) -> async_graphql::Result<
        async_graphql::Json<crystal_core::persistence::documents::ManifestDocument>,
    > {
        let context = ctx.data::<ManifestRequestContext>()?;
        let state = ctx.data::<crate::state::AppState>()?;
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
            .save(&state.db_pool, context, &identifier, &revision, envelope.0)
            .await
            .map(async_graphql::Json)
            .map_err(async_graphql::Error::new)
    }
    /// Append a typed operational observation for the active organization.
    async fn record_report(
        &self,
        ctx: &Context<'_>,
        manifest_id: String,
        report: async_graphql::Json<crystal_core::reports::Report>,
    ) -> async_graphql::Result<bool> {
        let pool = ctx.data::<DatabasePool>()?;
        let context = ctx.data::<ManifestRequestContext>()?;
        crystal_core::persistence::record_report(pool, context, &manifest_id, report.0)
            .await
            .map_err(async_graphql::Error::new)
    }

    /// Record one GitHub Actions observation without rewriting a generated Manifest.
    async fn record_action_run(
        &self,
        ctx: &Context<'_>,
        manifest_id: String,
        run: async_graphql::Json<crystal_core::action_history::GithubActionRun>,
        event_id: Option<String>,
        #[graphql(default = "api")] source: String,
    ) -> async_graphql::Result<bool> {
        let pool = ctx.data::<DatabasePool>()?;
        let context = ctx
            .data::<ManifestRequestContext>()
            .map_err(|_| async_graphql::Error::new("request is missing active organization"))?;
        crystal_core::persistence::record_action_run(
            pool,
            context,
            &manifest_id,
            run.0,
            event_id,
            &source,
        )
        .await
        .map_err(async_graphql::Error::new)
    }

    /// Upsert a generated manifest artifact into storage.
    async fn upsert_generated_manifest(
        &self,
        ctx: &Context<'_>,
        input: UpsertGeneratedManifestInput,
    ) -> async_graphql::Result<UpsertGeneratedManifestPayload> {
        let pool = ctx
            .data::<DatabasePool>()
            .map_err(|error| async_graphql::Error::new(error.message))?;
        let request_context = ctx
            .data::<ManifestRequestContext>()
            .map_err(|_| async_graphql::Error::new("request is missing active organization"))?;
        if !request_context.can_write_generated_manifests() {
            return Err(async_graphql::Error::new(
                "active organization role cannot upload generated manifests",
            ));
        }

        let id = persist_generated_manifest(pool, &input, request_context)
            .await
            .map_err(async_graphql::Error::new)?;

        Ok(UpsertGeneratedManifestPayload { id })
    }
}

#[cfg(test)]
mod tests;
