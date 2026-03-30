use std::time::Duration;

/// The primary error type for the Rusty AI SDK.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("Unsupported capability `{capability}` for provider `{provider}`")]
    UnsupportedCapability {
        capability: String,
        provider: String,
    },

    #[error("Provider `{provider}` error (status {status:?}): {message}")]
    ProviderError {
        provider: String,
        status: Option<u16>,
        message: String,
    },

    #[error("Platform `{platform}` is unavailable")]
    PlatformUnavailable { platform: String },

    #[error("Model `{model}` is unavailable")]
    ModelUnavailable { model: String },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Rate limited (retry after {retry_after:?})")]
    RateLimit { retry_after: Option<Duration> },

    #[error("Timeout")]
    Timeout,

    #[error("Request was cancelled")]
    Cancelled,

    #[error("Transport error: {message}")]
    Transport {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Tool `{tool_name}` error: {message}")]
    ToolError {
        tool_name: String,
        message: String,
    },

    #[error("Bridge `{bridge}` error: {message}")]
    BridgeError {
        bridge: String,
        message: String,
    },

    #[error("Schema validation error: {message}")]
    SchemaValidation { message: String },

    #[error("Stream error: {message}")]
    StreamError { message: String },

    #[error("Exceeded maximum steps ({max_steps})")]
    MaxStepsExceeded { max_steps: usize },
}

impl From<serde_json::Error> for AiError {
    fn from(err: serde_json::Error) -> Self {
        AiError::Serialization(err.to_string())
    }
}

/// Convenience result type alias.
pub type AiResult<T> = Result<T, AiError>;
