//! Auth command argument parsing.

use clap::{Args as ClapArgs, Subcommand};

/// Interactive auth commands.
#[derive(ClapArgs, Debug, Clone)]
pub(crate) struct AuthArgs {
    #[command(subcommand)]
    pub(crate) command: AuthCommand,
}

/// Supported auth subcommands.
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum AuthCommand {
    /// Open the browser and sign in through Clerk OAuth.
    Login(LoginArgs),
    /// Delete the stored Scryr CLI OAuth tokens.
    Logout,
    /// Show the currently authenticated Clerk subject.
    Whoami,
}

/// Browser-based Clerk OAuth login.
#[derive(ClapArgs, Debug, Clone)]
pub(crate) struct LoginArgs {
    /// Clerk publishable key used for OAuth discovery.
    #[arg(
        long,
        env = "SCRYR_CLERK_PUBLISHABLE_KEY",
        alias = "clerk-publishable-key"
    )]
    pub(crate) clerk_publishable_key: Option<String>,
    /// OAuth client id for the public Clerk OAuth application.
    #[arg(long, env = "SCRYR_CLERK_OAUTH_CLIENT_ID")]
    pub(crate) oauth_client_id: Option<String>,
    /// Space-delimited scopes requested during login.
    #[arg(
        long,
        env = "SCRYR_CLERK_OAUTH_SCOPES",
        default_value = "profile email"
    )]
    pub(crate) scopes: String,
}
