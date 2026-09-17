//! GraphQL root resolvers.

use crate::auth::require_authenticated_request;
use crate::generated_manifest_graphql::GeneratedManifestMutationRoot;
use crate::health::run_healthcheck;
use crate::state::AppState;
use async_graphql::Object;
use crystal_core::graphql_types::{Block, HealthStatus, ScryrMap};
use crystal_core::persistence;

/// Overlay stored collection health without changing a generated artifact or build status.
async fn attach_provider_sync(
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
    let statuses = persistence::read_provider_sync(pool, clerk_org_id, manifest_id).await?;
    if !statuses.is_empty() {
        raw_json
            .as_object_mut()
            .ok_or("generated manifest block must be an object")?
            .insert(
                "providerSync".into(),
                serde_json::to_value(statuses).map_err(|error| error.to_string())?,
            );
    }
    Ok(())
}

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
    let mut history =
        persistence::read_action_history(pool, clerk_org_id, manifest_id, 100, 0).await?;
    if history.runs.is_empty() {
        return Ok(());
    }
    history
        .runs
        .retain(|run| matches_action_source(raw_json, run));
    let last_build = history.runs.iter().map(|run| run.updated_at).max();
    let status = history.build_status();
    let cicd = raw_json
        .as_object_mut()
        .ok_or("generated manifest block must be an object")?
        .entry("cicd")
        .or_insert_with(|| serde_json::json!({}));
    let cicd = cicd
        .as_object_mut()
        .ok_or("generated manifest cicd section must be an object")?;
    let Some(last_build) = last_build else {
        for key in ["githubActions", "lastBuild", "buildStatus"] {
            cicd.remove(key);
        }
        return Ok(());
    };
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

/// Keep prior source subscriptions from appearing after a declaration changes.
fn matches_action_source(
    block: &serde_json::Value,
    run: &crystal_core::action_history::GithubActionRun,
) -> bool {
    let mut repository = None;
    if let Some(repo) = block["github"]["repoUrl"].as_str() {
        let Ok(repo) = url::Url::parse(repo) else {
            return false;
        };
        if !repo
            .host_str()
            .is_some_and(|host| host.eq_ignore_ascii_case(&run.host))
            || !repo
                .path()
                .trim_matches('/')
                .trim_end_matches(".git")
                .eq_ignore_ascii_case(&run.repository)
        {
            return false;
        }
        repository = Some(
            repo.path()
                .trim_matches('/')
                .trim_end_matches(".git")
                .to_owned(),
        );
    }
    let source = &block["cicd"]["source"];
    let selected_workflows = source["workflow_id"].as_u64().is_some()
        || source["workflows"]
            .as_array()
            .is_some_and(|names| !names.is_empty());
    let branch = source["branch"].as_str().or_else(|| {
        let sync_context = &block["providerSync"]["github"]["context"];
        if selected_workflows
            && repository
                .as_deref()
                .zip(sync_context["repository"].as_str())
                .is_some_and(|(configured, collected)| configured.eq_ignore_ascii_case(collected))
        {
            sync_context["defaultBranch"]
                .as_str()
                .filter(|name| !name.is_empty())
        } else {
            None
        }
    });
    if branch.is_some_and(|branch| run.head_branch.as_deref() != Some(branch))
        || source["workflow_id"]
            .as_u64()
            .is_some_and(|id| run.workflow_id != id)
    {
        return false;
    }
    if let Some(workflows) = source["workflows"]
        .as_array()
        .filter(|names| !names.is_empty())
    {
        let filename = run
            .workflow_path
            .as_deref()
            .and_then(|path| path.split('@').next())
            .and_then(|path| path.rsplit('/').next());
        if !workflows
            .iter()
            .any(|name| name.as_str().is_some() && name.as_str() == filename)
        {
            return false;
        }
    }
    true
}

/// Overlay successful dependency components only for the current declared repository.
async fn attach_github_dependencies(
    pool: &persistence::DatabasePool,
    clerk_org_id: &str,
    block: &mut serde_json::Value,
) -> Result<(), String> {
    // Runtime snapshots never survive a disabled or changed declaration.
    if let Some(dependencies) = block
        .get_mut("dependencies")
        .and_then(serde_json::Value::as_object_mut)
    {
        dependencies.remove("github");
    }
    let source = &block["dependencies"]["source"];
    if source["provider"].as_str() != Some("github") {
        return Ok(());
    }
    let Some(manifest_id) = block["manifestId"].as_str() else {
        return Ok(());
    };
    let Some(repository_url) = block["github"]["repoUrl"].as_str() else {
        return Ok(());
    };
    let Ok(repository_url) = url::Url::parse(repository_url) else {
        return Ok(());
    };
    if repository_url.scheme() != "https"
        || repository_url.host_str() != Some("github.com")
        || !repository_url.username().is_empty()
        || repository_url.password().is_some()
        || repository_url.port().is_some()
        || repository_url.query().is_some()
        || repository_url.fragment().is_some()
    {
        return Ok(());
    }
    let Some(mut snapshot) =
        persistence::read_github_dependencies(pool, clerk_org_id, manifest_id).await?
    else {
        return Ok(());
    };
    if !repository_url
        .path()
        .trim_matches('/')
        .trim_end_matches(".git")
        .eq_ignore_ascii_case(&snapshot.repository)
    {
        return Ok(());
    }
    if source["inventory"].as_bool() == Some(false) {
        snapshot.inventory = None;
    }
    if source["security"].as_bool() == Some(false) {
        snapshot.security = None;
    }
    if snapshot.inventory.is_none() && snapshot.security.is_none() {
        return Ok(());
    }
    block["dependencies"]["github"] =
        serde_json::to_value(snapshot).map_err(|error| error.to_string())?;
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
        crate::samples::ensure_samples(state, &context).await?;
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

        // Operational history is stored separately so reports never rewrite generated artifacts.
        // Enrich the response copy that the map already polls every 30 seconds.
        let mut blocks = Vec::new();
        if let Some(items) = raw_json.as_array() {
            for item in items {
                let mut item = item.clone();
                attach_provider_sync(pool, &request_context.clerk_org_id, &mut item).await?;
                attach_action_history(pool, &request_context.clerk_org_id, &mut item).await?;
                attach_github_dependencies(pool, &request_context.clerk_org_id, &mut item).await?;
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
    use super::{attach_action_history, attach_github_dependencies, attach_provider_sync};
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

        // An unsuccessful collection must be visible even without action history,
        // and must never replace an observed workflow outcome.
        persistence::record_provider_sync(
            &pool,
            &context,
            "services/api",
            "github",
            Some("offline".into()),
        )
        .await?;
        attach_provider_sync(&pool, "org", &mut block).await?;
        assert_eq!(block["providerSync"]["github"]["error"], "offline");
        assert!(block["providerSync"]["github"]["lastSuccessAt"].is_null());
        assert_eq!(block["cicd"]["buildStatus"], "passing");
        let mut minimal = serde_json::json!({"manifestId": "services/api"});
        attach_provider_sync(&pool, "org", &mut minimal).await?;
        assert_eq!(minimal["providerSync"]["github"]["error"], "offline");
        assert!(minimal.get("cicd").is_none());
        let mut other_tenant = serde_json::json!({"manifestId": "services/api"});
        attach_provider_sync(&pool, "other", &mut other_tenant).await?;
        assert!(other_tenant.get("providerSync").is_none());
        let mut legacy = serde_json::json!({"name": "legacy"});
        attach_provider_sync(&pool, "org", &mut legacy).await?;
        assert_eq!(legacy, serde_json::json!({"name": "legacy"}));

        sqlite.close().await;
        Ok(())
    }

    #[tokio::test]
    async fn history_tracks_current_source_selection() -> Result<(), Box<dyn std::error::Error>> {
        let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let pool = DatabasePool::Sqlite(sqlite);
        let context = ManifestRequestContext {
            clerk_user_id: "user".into(),
            clerk_org_id: "org".into(),
            clerk_org_slug: None,
            clerk_org_role: Some("org:admin".into()),
            clerk_org_permissions: vec![],
        };
        let mut run: GithubActionRun = serde_json::from_value(serde_json::json!({
            "host":"github.com", "repositoryId":123, "repository":"example/api",
            "workflowId":42, "workflowName":"CI", "runId":1, "runAttempt":1,
            "headBranch":"main", "headSha":"abcdef", "htmlUrl":"https://github.com/example/api/actions/runs/1",
            "status":"completed", "conclusion":"failure",
            "createdAt":"2026-09-08T10:00:00Z", "updatedAt":"2026-09-08T10:02:00Z"
        }))?;
        persistence::record_action_run(&pool, &context, "api", run.clone(), None, "api").await?;
        let original = serde_json::json!({"manifestId":"api", "github":{"repoUrl":"https://github.com/example/api.git"},
            "cicd":{"source":{"workflows":["ci.yml"], "branch":"main"}}});
        let mut block = original.clone();
        attach_action_history(&pool, "org", &mut block).await?;
        assert!(block["cicd"].get("githubActions").is_none()); // Filename is unknown in old history.
        run.workflow_path = Some(".github/workflows/ci.yml".into());
        persistence::record_action_run(&pool, &context, "api", run.clone(), None, "api").await?;
        attach_action_history(&pool, "org", &mut block).await?;
        assert_eq!(block["cicd"]["buildStatus"], "failing");
        for source in [
            serde_json::json!({"workflows":["ci.yml"], "branch":"develop"}),
            serde_json::json!({"workflows":["other.yml"], "branch":"main"}),
            serde_json::json!({"workflow_id":99, "branch":"main"}),
        ] {
            let mut changed = block.clone();
            changed["cicd"]["source"] = source;
            attach_action_history(&pool, "org", &mut changed).await?;
            for key in ["githubActions", "buildStatus", "lastBuild"] {
                assert!(changed["cicd"].get(key).is_none());
            }
        }
        let mut moved = block.clone();
        moved["github"]["repoUrl"] = serde_json::json!("https://github.com/other/api");
        attach_action_history(&pool, "org", &mut moved).await?;
        assert!(moved["cicd"].get("githubActions").is_none());
        run.run_id = 2;
        run.workflow_id = 99;
        run.workflow_path = Some(".github/workflows/integration.yml".into());
        run.conclusion = Some("success".into());
        persistence::record_action_run(&pool, &context, "api", run, None, "api").await?;
        block["cicd"]["source"]["workflows"] = serde_json::json!(["ci.yml", "integration.yml"]);
        attach_action_history(&pool, "org", &mut block).await?;
        assert_eq!(block["cicd"]["buildStatus"], "failing");
        assert_eq!(
            block["cicd"]["githubActions"]["runs"]
                .as_array()
                .map(Vec::len),
            Some(2)
        );
        block["cicd"]["source"] = serde_json::json!({"workflow_id":99});
        attach_action_history(&pool, "org", &mut block).await?;
        assert_eq!(block["cicd"]["buildStatus"], "passing");
        Ok(())
    }
    #[tokio::test]
    async fn selected_workflows_follow_collected_default_branch()
    -> Result<(), Box<dyn std::error::Error>> {
        let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let pool = DatabasePool::Sqlite(sqlite);
        let context = ManifestRequestContext {
            clerk_user_id: "user".into(),
            clerk_org_id: "org".into(),
            clerk_org_slug: None,
            clerk_org_role: Some("org:admin".into()),
            clerk_org_permissions: vec![],
        };
        for (id, branch, conclusion) in [(1, "main", "failure"), (2, "trunk", "success")] {
            let run: GithubActionRun = serde_json::from_value(serde_json::json!({
                "host":"github.com", "repositoryId":123, "repository":"example/api",
                "workflowId":42, "workflowName":"CI", "workflowPath":".github/workflows/ci.yml",
                "runId":id, "runAttempt":1, "headBranch":branch, "headSha":"abcdef",
                "htmlUrl":"https://github.com/example/api/actions/runs/1",
                "status":"completed", "conclusion":conclusion,
                "createdAt":"2026-09-08T10:00:00Z", "updatedAt":"2026-09-08T10:02:00Z"
            }))?;
            persistence::record_action_run(&pool, &context, "api", run, None, "api").await?;
        }
        let original = serde_json::json!({"manifestId":"api", "github":{"repoUrl":"https://github.com/example/api.git"},
            "cicd":{"source":{"workflows":["ci.yml"]}}});
        for (branch, expected) in [("main", "failing"), ("trunk", "passing")] {
            persistence::record_provider_sync_with_context(
                &pool,
                &context,
                "api",
                "github",
                None,
                Some(serde_json::json!({"repository":"example/api", "defaultBranch":branch})),
            )
            .await?;
            let mut block = original.clone();
            attach_provider_sync(&pool, "org", &mut block).await?;
            attach_action_history(&pool, "org", &mut block).await?;
            assert_eq!(block["cicd"]["buildStatus"], expected);
            assert_eq!(
                block["cicd"]["githubActions"]["runs"]
                    .as_array()
                    .map(Vec::len),
                Some(1)
            );
        }
        persistence::record_provider_sync(&pool, &context, "api", "github", Some("offline".into()))
            .await?;
        let mut block = original.clone();
        attach_provider_sync(&pool, "org", &mut block).await?;
        attach_action_history(&pool, "org", &mut block).await?;
        assert_eq!(block["cicd"]["buildStatus"], "passing");
        assert_eq!(block["providerSync"]["github"]["error"], "offline");
        block["cicd"]["source"] = serde_json::json!({"workflow_id":42});
        attach_action_history(&pool, "org", &mut block).await?;
        assert_eq!(block["cicd"]["buildStatus"], "passing");
        block["cicd"]["source"]["branch"] = serde_json::json!("main");
        attach_action_history(&pool, "org", &mut block).await?;
        assert_eq!(block["cicd"]["buildStatus"], "failing");
        Ok(())
    }

    #[test]
    fn default_branch_context_requires_matching_repository_and_selection()
    -> Result<(), Box<dyn std::error::Error>> {
        let run: GithubActionRun = serde_json::from_value(serde_json::json!({
            "host":"github.com", "repositoryId":123, "repository":"example/api",
            "workflowId":42, "workflowName":"CI", "workflowPath":".github/workflows/ci.yml",
            "runId":1, "runAttempt":1, "headBranch":"feature", "headSha":"abcdef",
            "htmlUrl":"https://github.com/example/api/actions/runs/1",
            "status":"completed", "conclusion":"success",
            "createdAt":"2026-09-08T10:00:00Z", "updatedAt":"2026-09-08T10:02:00Z"
        }))?;
        let mut block = serde_json::json!({"github":{"repoUrl":"https://github.com/example/api"},
            "cicd":{"source":{"workflows":["ci.yml"]}},
            "providerSync":{"github":{"context":{"repository":"other/api", "defaultBranch":"main"}}}});
        assert!(super::matches_action_source(&block, &run));
        block["providerSync"]["github"]["context"]["repository"] = serde_json::json!("EXAMPLE/API");
        assert!(!super::matches_action_source(&block, &run));
        block["cicd"]["source"] = serde_json::json!({});
        assert!(super::matches_action_source(&block, &run));
        block["cicd"]["source"] = serde_json::json!({"workflow_id":42});
        assert!(!super::matches_action_source(&block, &run));
        block
            .as_object_mut()
            .ok_or("block object")?
            .remove("github");
        assert!(super::matches_action_source(&block, &run));
        Ok(())
    }

    fn dependency_snapshot() -> serde_json::Value {
        serde_json::json!({"repository":"example/api",
            "inventory":{"observedAt":"2026-09-08T10:00:00Z", "packages":[{"id":"react18","name":"react","version":"18","license":"MIT","purl":"pkg:npm/react@18"}]},
            "security":{"observedAt":"2026-09-08T10:00:00Z", "alerts":[]}})
    }

    #[tokio::test]
    async fn dependencies_overlay_requires_current_opt_in_repository_and_component()
    -> Result<(), Box<dyn std::error::Error>> {
        let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let pool = DatabasePool::Sqlite(sqlite);
        let context = ManifestRequestContext {
            clerk_user_id: "user".into(),
            clerk_org_id: "org".into(),
            clerk_org_slug: None,
            clerk_org_role: Some("org:admin".into()),
            clerk_org_permissions: vec![],
        };
        persistence::record_github_dependencies(
            &pool,
            &context,
            "api",
            serde_json::from_value(dependency_snapshot())?,
        )
        .await?;
        let original = serde_json::json!({"manifestId":"api", "github":{"repoUrl":"https://github.com/Example/API.git/"},
            "dependencies":{"source":{"provider":"github"}, "totalDeps":123, "vulnerableDeps":2}});
        let mut block = original.clone();
        attach_github_dependencies(&pool, "org", &mut block).await?;
        assert_eq!(block["dependencies"]["github"], dependency_snapshot());
        assert_eq!(block["dependencies"]["totalDeps"], 123);
        assert_eq!(block["dependencies"]["vulnerableDeps"], 2);
        for (inventory, security) in [(false, true), (true, false), (false, false)] {
            let mut disabled = block.clone();
            disabled["dependencies"]["source"]["inventory"] = serde_json::json!(inventory);
            disabled["dependencies"]["source"]["security"] = serde_json::json!(security);
            attach_github_dependencies(&pool, "org", &mut disabled).await?;
            assert_eq!(
                disabled["dependencies"]["github"]
                    .get("inventory")
                    .is_some(),
                inventory
            );
            assert_eq!(
                disabled["dependencies"]["github"].get("security").is_some(),
                security
            );
        }
        for repository in [
            "https://github.com/other/api",
            "https://github.enterprise/example/api",
            "https://evil.example/example/api",
            "https://github.com/example/api?token=x",
        ] {
            let mut moved = block.clone();
            moved["github"]["repoUrl"] = serde_json::json!(repository);
            attach_github_dependencies(&pool, "org", &mut moved).await?;
            assert!(moved["dependencies"].get("github").is_none());
        }
        let mut disabled = block.clone();
        disabled["dependencies"]["source"] = serde_json::Value::Null;
        attach_github_dependencies(&pool, "org", &mut disabled).await?;
        assert!(disabled["dependencies"].get("github").is_none());
        let mut other_tenant = block.clone();
        attach_github_dependencies(&pool, "other", &mut other_tenant).await?;
        assert!(other_tenant["dependencies"].get("github").is_none());
        // Collection failures update provider health only; a successful prior snapshot remains visible.
        persistence::record_provider_sync(
            &pool,
            &context,
            "api",
            "github_dependencies_security",
            Some("offline".into()),
        )
        .await?;
        attach_provider_sync(&pool, "org", &mut block).await?;
        attach_github_dependencies(&pool, "org", &mut block).await?;
        assert_eq!(block["dependencies"]["github"], dependency_snapshot());
        assert_eq!(
            block["providerSync"]["github_dependencies_security"]["error"],
            "offline"
        );
        Ok(())
    }

    #[tokio::test]
    async fn dependency_mutation_validates_permissions_and_snapshot_wire_shape()
    -> Result<(), Box<dyn std::error::Error>> {
        use async_graphql::{EmptySubscription, Request, Schema, Variables};
        let sqlite = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let pool = DatabasePool::Sqlite(sqlite);
        let schema = Schema::build(
            super::QueryRoot,
            super::MutationRoot::default(),
            EmptySubscription,
        )
        .data(pool.clone())
        .finish();
        let request = || {
            Request::new("mutation($snapshot:JSON!){recordGithubDependencies(manifestId:\"api\",snapshot:$snapshot)}")
            .variables(Variables::from_json(serde_json::json!({"snapshot":dependency_snapshot()})))
        };
        assert!(!schema.execute(request()).await.errors.is_empty());
        let mut principal = ManifestRequestContext {
            clerk_user_id: "user".into(),
            clerk_org_id: "org".into(),
            clerk_org_slug: None,
            clerk_org_role: None,
            clerk_org_permissions: vec![],
        };
        assert!(
            !schema
                .execute(request().data(principal.clone()))
                .await
                .errors
                .is_empty()
        );
        principal.clerk_org_role = Some("org:admin".into());
        let response = schema.execute(request().data(principal.clone())).await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        assert_eq!(response.data.into_json()?["recordGithubDependencies"], true);
        assert_eq!(
            schema
                .execute(request().data(principal.clone()))
                .await
                .data
                .into_json()?["recordGithubDependencies"],
            false
        );
        let bad = Request::new("mutation{recordGithubDependencies(manifestId:\"api\",snapshot:{repository:\"example/api\"})}").data(principal);
        assert!(!schema.execute(bad).await.errors.is_empty());
        assert!(
            persistence::read_github_dependencies(&pool, "other", "api")
                .await?
                .is_none()
        );
        Ok(())
    }
}
