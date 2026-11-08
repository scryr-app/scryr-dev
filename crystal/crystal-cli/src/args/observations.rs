//! Operational reporter command arguments.
#![allow(clippy::missing_docs_in_private_items)]
use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Debug, Args)]
pub(crate) struct ReportsArgs {
    #[command(subcommand)]
    pub command: ReportCommand,
}
#[derive(Clone, Debug, Subcommand)]
pub(crate) enum ReportCommand {
    /// Collect GitHub builds, releases and deployments configured in index.scry.
    Delivery(DeliveryArgs),
    /// Report a GitHub workflow event.
    Actions(super::ReportArgs),
    /// Import `JUnit` XML results from one complete suite/shard.
    Tests(ObservationArgs),
    /// Import Cobertura XML or LCOV line coverage.
    Coverage(ObservationArgs),
    /// Collect a complete Dependabot snapshot, or import an offline JSON array.
    Dependencies(ObservationArgs),
    /// Record a deployment for one environment.
    Deployment(ObservationArgs),
}

#[derive(Clone, Debug, Args)]
pub(crate) struct DeliveryArgs {
    #[command(flatten)]
    pub source: super::ReportSourceArgs,
    /// Stable manifest identity; configuration is still loaded from index.scry.
    #[arg(long, default_value = "")]
    pub manifest_id: String,
    #[arg(long, alias = "graphql-url", env = "SCRYR_ENDPOINT", default_value_t = super::workflow::default_endpoint())]
    pub endpoint: String,
    #[arg(long, env = "SCRYR_CLERK_ORG_ID")]
    pub clerk_org_id: Option<String>,
    /// Import a previously collected, typed delivery snapshot instead of contacting GitHub.
    #[arg(long)]
    pub file: Option<PathBuf>,
    /// Typed action declared in delivery.tasks (defaults to collect).
    #[arg(long, value_enum, default_value_t = DeliveryTaskName::Collect)]
    pub task: DeliveryTaskName,
    /// Override the selected task's maximum recent records per GitHub source.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=100))]
    pub limit: Option<u32>,
    /// Download release assets and verify their digests and build-workflow attestations.
    #[arg(long, conflicts_with = "file")]
    pub verify_attestations: bool,
    /// Print the validated report without uploading it.
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum DeliveryTaskName {
    Collect,
    Verify,
}

impl DeliveryArgs {
    pub(crate) fn resolve_task(
        &self,
        tasks: &crystal_core::delivery::DeliveryTasks,
    ) -> Result<crystal_core::delivery::DeliveryTask, String> {
        if self.file.is_some() && self.task == DeliveryTaskName::Verify {
            return Err("--task verify cannot be combined with --file; offline import does not verify attestations".into());
        }
        let mut task = match self.task {
            DeliveryTaskName::Collect => tasks.collect.clone(),
            DeliveryTaskName::Verify => tasks.verify.clone(),
        };
        if let Some(limit) = self.limit {
            task.limit = limit;
        }
        task.verify_attestations |= self.verify_attestations;
        if self.file.is_some() && task.verify_attestations {
            return Err(
                "--file cannot be combined with a task configured to verify attestations".into(),
            );
        }
        Ok(task)
    }
}
#[derive(Clone, Debug, Args)]
pub(crate) struct ObservationArgs {
    #[command(flatten)]
    pub source: super::ReportSourceArgs,
    #[arg(long, default_value = "")]
    pub manifest_id: String,
    #[arg(
        long,
        alias = "graphql-url",
        env = "SCRYR_ENDPOINT",
        default_value_t = super::workflow::default_endpoint()
    )]
    pub endpoint: String,
    #[arg(long, env = "SCRYR_CLERK_ORG_ID")]
    pub clerk_org_id: Option<String>,
    #[arg(long)]
    pub file: Vec<PathBuf>,
    #[arg(long)]
    pub format: Option<String>,
    #[arg(long, alias = "scope")]
    pub suite: Option<String>,
    #[arg(long)]
    pub shard: Option<String>,
    #[arg(long, env = "GITHUB_RUN_ID")]
    pub run_id: String,
    #[arg(long, env = "GITHUB_RUN_ATTEMPT", default_value_t = 1)]
    pub attempt: u32,
    #[arg(long, env = "GITHUB_SHA")]
    pub commit_sha: Option<String>,
    #[arg(long, env = "GITHUB_REF_NAME")]
    pub branch: Option<String>,
    /// RFC3339 source completion/snapshot time; required for retry-stable delivery.
    #[arg(long)]
    pub observed_at: String,
    #[arg(long)]
    pub report_url: Option<String>,
    #[arg(long)]
    pub repository: Option<String>,
    #[arg(long)]
    pub manifest_path: Option<String>,
    #[arg(long, default_value = "dependabot")]
    pub provider: String,
    #[arg(long)]
    pub environment: Option<String>,
    #[arg(long)]
    pub status: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use crystal_core::delivery::{DeliveryTask, DeliveryTasks};

    #[derive(Parser)]
    struct DeliveryCli {
        #[command(flatten)]
        delivery: DeliveryArgs,
    }

    fn parse(arguments: &[&str]) -> Result<DeliveryArgs, clap::Error> {
        DeliveryCli::try_parse_from(std::iter::once("delivery").chain(arguments.iter().copied()))
            .map(|parsed| parsed.delivery)
    }

    #[test]
    fn delivery_actions_resolve_declared_defaults_and_cli_overrides()
    -> Result<(), Box<dyn std::error::Error>> {
        let tasks = DeliveryTasks {
            collect: DeliveryTask {
                limit: 7,
                verify_attestations: false,
            },
            verify: DeliveryTask {
                limit: 9,
                verify_attestations: true,
            },
        };
        assert_eq!(parse(&[])?.resolve_task(&tasks)?, tasks.collect);
        assert_eq!(
            parse(&["--task", "verify"])?.resolve_task(&tasks)?,
            tasks.verify
        );
        assert_eq!(
            parse(&[
                "--task",
                "collect",
                "--limit",
                "12",
                "--verify-attestations"
            ])?
            .resolve_task(&tasks)?,
            DeliveryTask {
                limit: 12,
                verify_attestations: true
            }
        );
        assert_eq!(
            parse(&["--task", "verify", "--limit", "3"])?.resolve_task(&tasks)?,
            DeliveryTask {
                limit: 3,
                verify_attestations: true
            }
        );
        Ok(())
    }

    #[test]
    fn task_configuration_can_disable_default_verification()
    -> Result<(), Box<dyn std::error::Error>> {
        let tasks: DeliveryTasks =
            serde_json::from_str(r#"{"verify":{"verifyAttestations":false}}"#)?;
        assert!(
            !parse(&["--task", "verify"])?
                .resolve_task(&tasks)?
                .verify_attestations
        );
        assert!(
            parse(&["--task", "verify", "--verify-attestations"])?
                .resolve_task(&tasks)?
                .verify_attestations
        );
        Ok(())
    }

    #[test]
    fn offline_import_does_not_run_a_verification_action() -> Result<(), Box<dyn std::error::Error>>
    {
        let tasks = DeliveryTasks::default();
        assert!(
            parse(&["--file", "snapshot.json"])?
                .resolve_task(&tasks)
                .is_ok()
        );
        assert!(
            parse(&["--task", "verify", "--file", "snapshot.json"])?
                .resolve_task(&tasks)
                .is_err()
        );
        assert!(parse(&["--verify-attestations", "--file", "snapshot.json"]).is_err());
        assert!(parse(&["--task", "custom-shell-command"]).is_err());
        let mut verifying_collect = tasks;
        verifying_collect.collect.verify_attestations = true;
        assert!(
            parse(&["--file", "snapshot.json"])?
                .resolve_task(&verifying_collect)
                .is_err()
        );
        for value in ["0", "101"] {
            assert!(parse(&["--limit", value]).is_err());
        }
        Ok(())
    }
}
