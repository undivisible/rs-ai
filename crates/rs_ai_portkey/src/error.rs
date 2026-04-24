use rs_ai_ai::AiError;

#[derive(Debug, thiserror::Error)]
pub enum PortkeyError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Portkey API error: {message}")]
    ApiError { message: String },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Streaming error: {0}")]
    StreamError(String),
}

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
