//! Error types for LangFuse integration.

use rs_ai_core::AiError;

/// Result type for LangFuse operations.
pub type LangfuseResult<T> = Result<T, LangfuseError>;

/// Errors that can occur during LangFuse observability operations.
#[derive(Debug, thiserror::Error)]
pub enum LangfuseError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    /// JSON serialization failed.
    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// LangFuse API returned an error.
    #[error("LangFuse API error: {status} {message}")]
    ApiError {
        /// HTTP status code.
        status: u16,
        /// Error message.
        message: String,
    },

    /// Failed to emit trace.
    #[error("Failed to emit trace: {0}")]
    EmitFailed(String),
}

impl From<LangfuseError> for AiError {
    fn from(err: LangfuseError) -> Self {
        AiError::ProviderError {
            provider: "langfuse".to_string(),
            status: None,
            message: err.to_string(),
        }
    }
}
