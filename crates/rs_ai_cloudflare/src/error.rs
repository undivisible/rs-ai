use rs_ai_traits::AiError;

/// Errors that can occur when calling the Cloudflare Workers AI API.
#[derive(Debug, thiserror::Error)]
pub enum CloudflareError {
    /// HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error returned by the Cloudflare API.
    #[error("Cloudflare API error: {message}")]
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

/// Shorthand result type for Cloudflare operations.
pub type CloudflareResult<T> = Result<T, CloudflareError>;

impl From<CloudflareError> for AiError {
    fn from(err: CloudflareError) -> Self {
        AiError::ProviderError {
            provider: "cloudflare".to_string(),
            status: None,
            message: err.to_string(),
        }
    }
}
