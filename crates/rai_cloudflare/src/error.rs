use rai_ai::AiError;

#[derive(Debug, thiserror::Error)]
pub enum CloudflareError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Cloudflare API error: {message}")]
    ApiError { message: String },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Streaming error: {0}")]
    StreamError(String),
}

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
