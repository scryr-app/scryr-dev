//! Scryr-managed `uv` installation and resolution.
#![allow(clippy::redundant_pub_crate)]

mod install;
mod paths;
mod version;

pub(crate) use install::ensure_managed_uv;
pub(crate) use paths::ManagedUv;

/// `uv` version owned by Scryr for manifest execution.
pub(crate) const REQUIRED_UV_VERSION: &str = "0.12.7";
