//! CLI argument parsing.
#![allow(clippy::missing_docs_in_private_items, clippy::redundant_pub_crate)]

mod observations;
pub(crate) use observations::{ObservationArgs, ReportCommand, ReportsArgs};
mod auth;
mod report;
pub(crate) use report::ReportArgs;
mod generate;
mod serve;
mod workflow;
pub(crate) use generate::{GenerateCommonArgs, resolve_generate_request};
pub(crate) use workflow::*;

pub(crate) use auth::{AuthArgs, AuthCommand, LoginArgs};
pub(crate) use generate::{GenerateArgs, GenerateOutput, GenerateRequest};
pub(crate) use serve::ServerArgs;

use clap::{Parser, Subcommand};

/// CLI arguments for manifest artifact generation and interactive auth.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "scryr",
    author,
    version,
    about,
    long_about = None,
    after_help = "Examples:\n  scryr check\n  scryr format\n  scryr lint --fix\n  scryr push\n  scryr serve\n  scryr report tests\n  scryr query --list\nLegacy commands:\n  scryr generate upload --path index.scry\n  scryr generate types --path index.scry\n  scryr generate mise --path index.scry --forge \"MERN Forge\"\n  scryr generate compose --path index.scry --forge \"MERN Forge\"\n  scryr generate devcontainer --path index.scry --forge \"MERN Forge\""
)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// Top-level CLI commands.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Command {
    /// Check formatting, lint, Python types, execution, and Scryr rules.
    Check(GenerateCommonArgs),
    /// Format manifest source files in place.
    Format(FormatArgs),
    /// Lint manifest sources; optionally apply safe fixes.
    Lint(LintArgs),
    /// Check and push diagrams to a Scryr deployment.
    Push(GenerateCommonArgs),
    /// Start the local UI, format, check, and load index.scry.
    Serve(ServerArgs),
    /// Report operational results.
    Report(Box<ReportsArgs>),
    /// Run a named query declared in index.scry.
    Query(QueryArgs),
    /// Export JSON or Forge configuration.
    Export(ExportArgs),
    /// Inspect model schemas and runtime type metadata.
    Inspect(InspectArgs),
    /// Apply database schema migrations without starting the HTTP server.
    Migrate,
    /// Convert an existing JSON connection file to TOML without overwriting either file.
    MigrateSecrets(MigrateSecretsArgs),
    /// Report a GitHub workflow run to Crystal.
    ReportActionStatus(ReportArgs),
    /// Generate manifest artifacts and persist them through GraphQL.
    Generate(Box<GenerateArgs>),
    /// Interactive Clerk authentication helpers.
    Auth(AuthArgs),
}

/// Fully resolved top-level command consumed by execution.
#[derive(Debug, Clone)]
pub(crate) enum ResolvedCommand {
    /// Check formatting, lint, Python types, execution, and Scryr rules.
    Check(GenerateCommonArgs),
    /// Format manifest source files in place.
    Format(FormatArgs),
    /// Lint manifest sources; optionally apply safe fixes.
    Lint(LintArgs),
    /// Check and push diagrams to a Scryr deployment.
    Push(GenerateCommonArgs),
    /// Start the local UI, format, check, and load index.scry.
    Serve(ServerArgs),
    /// Export JSON or Forge configuration.
    Export(ExportArgs),
    /// Inspect model schemas and runtime type metadata.
    Inspect(InspectArgs),
    /// Run a named query declared in index.scry.
    Query(QueryArgs),
    /// Apply database schema migrations without starting the HTTP server.
    Migrate,
    /// Convert an existing JSON connection file to TOML without overwriting either file.
    MigrateSecrets(MigrateSecretsArgs),
    /// Report a GitHub workflow run to Crystal.
    ReportActionStatus(ReportArgs),
    /// Report operational results.
    Report(Box<ReportsArgs>),
    /// Generate manifest artifacts or render one generated artifact.
    Generate(GenerateRequest),
    /// Interactive Clerk authentication helpers.
    Auth(AuthArgs),
}

impl Args {
    /// Convert CLI input into a concrete command.
    pub(crate) fn resolved_command(&self) -> Result<ResolvedCommand, String> {
        if let Some(command) = &self.command {
            return Ok(match command.clone() {
                Command::Check(args) => ResolvedCommand::Check(args),
                Command::Format(args) => ResolvedCommand::Format(args),
                Command::Lint(args) => ResolvedCommand::Lint(args),
                Command::Push(args) => ResolvedCommand::Push(args),
                Command::Export(args) => ResolvedCommand::Export(args),
                Command::Inspect(args) => ResolvedCommand::Inspect(args),
                Command::Query(args) => ResolvedCommand::Query(args),
                Command::Report(args) => ResolvedCommand::Report(args),
                Command::ReportActionStatus(args) => ResolvedCommand::ReportActionStatus(args),
                Command::Migrate => ResolvedCommand::Migrate,
                Command::MigrateSecrets(args) => ResolvedCommand::MigrateSecrets(args),
                Command::Serve(args) => ResolvedCommand::Serve(args),
                Command::Generate(args) => ResolvedCommand::Generate((*args).into_request()),
                Command::Auth(args) => ResolvedCommand::Auth(args),
            });
        }

        Err(
            "missing command: use `check`, `format`, `lint`, `push`, `serve`, `report`, or `query`"
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, GenerateOutput, ResolvedCommand};
    use clap::Parser;
    use crystal_server::state::AuthMode;
    use std::path::PathBuf;

    #[test]
    fn generate_command_accepts_command_local_scryr_dir() -> Result<(), String> {
        let args = Args::parse_from([
            "scryr",
            "generate",
            "upload",
            "--path",
            "sample.py",
            "--scryr-dir",
            "/tmp/scryr-state",
        ]);

        let ResolvedCommand::Generate(generate_args) = args.resolved_command()? else {
            return Err("expected generate command".to_string());
        };

        assert_eq!(
            generate_args.scryr_dir,
            Some(PathBuf::from("/tmp/scryr-state"))
        );
        Ok(())
    }

    #[test]
    fn generate_command_accepts_command_local_manifest_dir() -> Result<(), String> {
        let args = Args::parse_from([
            "scryr",
            "generate",
            "upload",
            "--path",
            "sample.py",
            "--manifest-dir",
            "/tmp/manifest",
        ]);

        let ResolvedCommand::Generate(generate_args) = args.resolved_command()? else {
            return Err("expected generate command".to_string());
        };

        assert_eq!(generate_args.manifest_dir, PathBuf::from("/tmp/manifest"));
        Ok(())
    }

    #[test]
    fn generate_command_defaults_to_index_scry() -> Result<(), String> {
        let args = Args::parse_from(["scryr", "generate", "upload"]);

        let ResolvedCommand::Generate(generate_args) = args.resolved_command()? else {
            return Err("expected generate command".to_string());
        };

        assert_eq!(generate_args.manifest_file, PathBuf::from("index.scry"));
        assert_eq!(generate_args.output, GenerateOutput::Upload);
        Ok(())
    }

    #[test]
    fn serve_command_does_not_default_to_a_sample() -> Result<(), String> {
        let args = Args::parse_from(["scryr", "serve"]);

        let ResolvedCommand::Serve(serve_args) = args.resolved_command()? else {
            return Err("expected serve command".to_string());
        };

        assert_eq!(serve_args.sample, None);
        Ok(())
    }

    #[test]
    fn serve_command_accepts_server_options() -> Result<(), String> {
        let args = Args::parse_from([
            "scryr",
            "serve",
            "--sample",
            "plane",
            "--host",
            "0.0.0.0",
            "--port",
            "9000",
            "--auth-mode",
            "clerk",
        ]);

        let ResolvedCommand::Serve(serve_args) = args.resolved_command()? else {
            return Err("expected serve command".to_string());
        };

        assert_eq!(serve_args.sample, Some("plane".to_string()));
        assert_eq!(serve_args.host, "0.0.0.0");
        assert_eq!(serve_args.port, 9000);
        assert_eq!(serve_args.auth_mode, Some(AuthMode::Clerk));
        Ok(())
    }
}
