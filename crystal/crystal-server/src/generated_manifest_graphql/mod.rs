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
