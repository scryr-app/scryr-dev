//! Application state and CLI configuration.

use clap::{Args as ClapArgs, Parser, ValueEnum};
use clerk_rs::{
    clerk::Clerk, validators::authorizer::ClerkAuthorizer,
    validators::jwks::MemoryCacheJwksProvider,
};
use crystal_core::persistence::DatabasePool;

/// Standalone `crystal-server` binary arguments.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Shared server arguments.
    #[command(flatten)]
    pub server: ServerArgs,
}

/// GraphQL server arguments accepted by the standalone binary and `scryr serve`.
#[derive(ClapArgs, Debug, Clone)]
pub struct ServerArgs {
    /// Subfolder name inside samples/ to serve (e.g. "plane", "calcom").
    /// Omit to serve all samples.
    #[arg(short, long, default_value = "mern")]
    pub sample: Option<String>,
    /// Host interface to bind.
    #[arg(long, env = "HOST", default_value = "127.0.0.1")]
    pub host: String,
    /// TCP port to bind.
    #[arg(long, env = "PORT", default_value_t = 8000)]
    pub port: u16,
    /// Authentication provider mode. Defaults to local for zero-config offline use.
    #[arg(long, env = "AUTH_MODE")]
    pub auth_mode: Option<AuthMode>,
}

/// Application state containing shared configuration.
#[derive(Clone)]
pub(crate) struct AppState {
    /// Optional samples subfolder name to serve (e.g. "plane").
    pub sample: Option<String>,
    /// Authentication provider mode for incoming requests.
    pub auth_mode: AuthMode,
    /// Clerk request authorizer backed by Clerk JWKS when configured.
    pub clerk_authorizer: Option<ClerkAuthorizer<MemoryCacheJwksProvider>>,
    /// Clerk Backend API client used for server-side membership verification.
    pub clerk_client: Option<Clerk>,
    /// Shared database connection pool used for health checks and generated manifests.
    pub db_pool: DatabasePool,
}

/// Authentication provider selected for this server process.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum AuthMode {
    /// Zero-config local development principal.
    Local,
    /// Hosted/cloud authentication through Clerk.
    Clerk,
}

impl AuthMode {
    /// Resolve auth mode from parsed config, defaulting to zero-config local auth.
    pub(crate) const fn resolve(configured: Option<Self>, _host: &str) -> Self {
        if let Some(auth_mode) = configured {
            return auth_mode;
        }

        Self::Local
    }

    /// Return a stable name for startup logging.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Clerk => "clerk",
        }
    }
}

/// Return whether a host is a loopback interface.
#[cfg(test)]
pub(crate) fn host_allows_zero_config_local_auth(host: &str) -> bool {
    let normalized = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_ascii_lowercase();

    matches!(normalized.as_str(), "localhost" | "127.0.0.1" | "::1")
}

#[cfg(test)]
mod tests {
    use super::{AuthMode, host_allows_zero_config_local_auth};

    #[test]
    fn unset_auth_mode_defaults_to_local_for_offline_use() {
        assert_eq!(AuthMode::resolve(None, "127.0.0.1"), AuthMode::Local);
        assert_eq!(AuthMode::resolve(None, "0.0.0.0"), AuthMode::Local);
        assert_eq!(AuthMode::resolve(None, "scryr.app"), AuthMode::Local);
    }

    #[test]
    fn configured_auth_mode_is_honored() {
        assert_eq!(
            AuthMode::resolve(Some(AuthMode::Clerk), "127.0.0.1"),
            AuthMode::Clerk
        );
    }

    #[test]
    fn loopback_host_detection_identifies_local_interfaces() {
        assert!(host_allows_zero_config_local_auth("localhost"));
        assert!(host_allows_zero_config_local_auth("127.0.0.1"));
        assert!(host_allows_zero_config_local_auth("[::1]"));

        assert!(!host_allows_zero_config_local_auth("0.0.0.0"));
        assert!(!host_allows_zero_config_local_auth("scryr.app"));
    }
}
