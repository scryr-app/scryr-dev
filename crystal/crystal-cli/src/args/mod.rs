//! CLI argument parsing.
#![allow(clippy::missing_docs_in_private_items, clippy::redundant_pub_crate)]

mod observations;
pub(crate) use observations::{ObservationArgs, ReportCommand, ReportsArgs};
mod auth;
mod report;
pub(crate) use report::ReportArgs;
mod generate;
mod serve;

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
    after_help = "Examples:\n  scryr serve\n  scryr generate upload --path index.scry\n  scryr generate types --path index.scry\n  scryr generate mise --path index.scry --forge \"MERN Forge\"\n  scryr generate compose --path index.scry --forge \"MERN Forge\"\n  scryr generate devcontainer --path index.scry --forge \"MERN Forge\""
)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

/// Top-level CLI commands.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Command {
    /// Run the Scryr GraphQL server.
    Serve(ServerArgs),
    /// Apply database schema migrations without starting the HTTP server.
    Migrate,
    /// Report a GitHub workflow run to Crystal.
    ReportActionStatus(ReportArgs),
    /// Report operational results.
    Report(Box<ReportsArgs>),
    /// Generate manifest artifacts and persist them through GraphQL.
    Generate(Box<GenerateArgs>),
    /// Interactive Clerk authentication helpers.
    Auth(AuthArgs),
}

/// Fully resolved top-level command consumed by execution.
#[derive(Debug, Clone)]
pub(crate) enum ResolvedCommand {
    /// Run the Scryr GraphQL server.
    Serve(ServerArgs),
    /// Apply database schema migrations without starting the HTTP server.
    Migrate,
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
                Command::Report(args) => ResolvedCommand::Report(args),
                Command::ReportActionStatus(args) => ResolvedCommand::ReportActionStatus(args),
                Command::Migrate => ResolvedCommand::Migrate,
                Command::Serve(args) => ResolvedCommand::Serve(args),
                Command::Generate(args) => ResolvedCommand::Generate((*args).into_request()),
                Command::Auth(args) => ResolvedCommand::Auth(args),
            });
        }

        Err(
            "missing command: use `serve`, `migrate`, `generate <target>`, or `auth <subcommand>`"
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

    #[test]
    fn generate_command_accepts_command_local_sprite_options() -> Result<(), String> {
        let args = Args::parse_from([
            "scryr",
            "generate",
            "types",
            "--path",
            "sample.py",
            "--sprite",
            "manifest-sandbox",
            "--sprite-org",
            "test-org",
        ]);

        let ResolvedCommand::Generate(generate_args) = args.resolved_command()? else {
            return Err("expected generate command".to_string());
        };

        assert_eq!(generate_args.sprite, Some("manifest-sandbox".to_string()));
        assert_eq!(generate_args.sprite_org, Some("test-org".to_string()));
        assert_eq!(generate_args.output, GenerateOutput::Types);
        Ok(())
    }
}
