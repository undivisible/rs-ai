use rs_ai_ai::AiError;

#[derive(Debug, thiserror::Error)]
pub enum XaiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("xAI API error: {message}")]
    ApiError { message: String },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Streaming error: {0}")]
    StreamError(String),
}

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
