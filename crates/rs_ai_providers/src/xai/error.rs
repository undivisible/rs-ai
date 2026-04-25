use rs_ai_core::AiError;

/// Errors that can occur when calling the xAI API.
#[derive(Debug, thiserror::Error)]
pub enum XaiError {
    /// HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error returned by the xAI API.
    #[error("xAI API error: {message}")]
    ApiError {
        /// Human-readable error message.
        message: String,
    },

    /// Authentication error.
    #[error("Authentication error: {message}")]
    AuthError {
        /// Human-readable error message.
        message: String,
    },

    /// Streaming error.
    #[error("Streaming error: {0}")]
    StreamError(String),
}

/// Shorthand result type for xAI operations.
pub type XaiResult<T> = Result<T, XaiError>;

impl From<XaiError> for AiError {
    fn from(err: XaiError) -> Self {
        AiError::ProviderError {
            provider: "xai".to_string(),
            status: None,
            message: err.to_string(),
        }
    }
}
