//! SDK-wide error type.

use thiserror::Error;

/// Result alias used throughout the SDK.
pub type SdkResult<T> = Result<T, SdkError>;

/// Top-level error type for all SDK operations.
#[derive(Debug, Error)]
pub enum SdkError {
    /// Attempted to use a brain that wasn't configured for the operation.
    #[error("brain is not configured for {feature}; rebuild with that feature enabled")]
    FeatureDisabled {
        /// Which feature was missing.
        feature: &'static str,
    },

    /// Serialization or deserialization failed.
    #[error("persistence error: {0}")]
    Persistence(String),

    /// Input validation failed.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Configuration validation failed in the builder.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Serde / JSON passthrough.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic I/O.
    #[error("io error: {0}")]
    Io(String),
}

impl From<std::io::Error> for SdkError {
    fn from(e: std::io::Error) -> Self {
        SdkError::Io(e.to_string())
    }
}