//! Auth command routing.

use crate::args::{AuthArgs, AuthCommand};
use crate::auth;

/// Execute an auth subcommand.
pub(super) async fn run(args: &AuthArgs) -> Result<(), String> {
    match args.command.clone() {
        AuthCommand::Login(login_args) => auth::login(&login_args).await,
        AuthCommand::Logout => auth::logout(),
        AuthCommand::Whoami => auth::whoami().await,
    }
}
