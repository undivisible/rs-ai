use rs_ai_core::AiError;

/// Errors that can occur when using the Portkey API.
#[derive(Debug, thiserror::Error)]
pub enum PortkeyError {
    /// An HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An error returned by the Portkey API.
    #[error("Portkey API error: {message}")]
    ApiError {
        /// Error message from the API.
        message: String,
    },

    /// An authentication error.
    #[error("Authentication error: {message}")]
    AuthError {
        /// Error message.
        message: String,
    },

    /// A streaming error.
    #[error("Streaming error: {0}")]
    StreamError(String),
}

/// Shorthand result type for Portkey operations.
pub type PortkeyResult<T> = Result<T, PortkeyError>;

impl From<PortkeyError> for AiError {
    fn from(err: PortkeyError) -> Self {
        AiError::ProviderError {
            provider: "portkey".to_string(),
            status: None,
            message: err.to_string(),
        }
    }
}
