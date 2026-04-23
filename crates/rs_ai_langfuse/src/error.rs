//! Error types for LangFuse integration.

use rai_ai::AiError;

/// Result type for LangFuse operations.
pub type LangfuseResult<T> = Result<T, LangfuseError>;

/// Errors that can occur during LangFuse observability operations.
#[derive(Debug, thiserror::Error)]
pub enum LangfuseError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("LangFuse API error: {status} {message}")]
    ApiError { status: u16, message: String },

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
