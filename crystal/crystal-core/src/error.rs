//! Errors exposed by shared generation and persistence APIs.

/// A failure in shared manifest processing or storage.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A manifest or generation configuration is invalid.
    Validation(String),
    /// A storage operation failed.
    Storage(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(message) | Self::Storage(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

// CLI and transport boundaries can preserve existing human-readable messages.
impl From<Error> for String {
    fn from(error: Error) -> Self {
        error.to_string()
    }
}
