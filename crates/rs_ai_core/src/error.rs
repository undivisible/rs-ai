//! Error types and result alias.
use std::time::Duration;

/// The primary error type for the Rusty AI SDK.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    /// The requested capability is not supported.
    #[error("Unsupported capability `{capability}` for provider `{provider}`")]
    UnsupportedCapability {
        /// Name of the unsupported capability.
        capability: String,
        /// ID of the provider that lacks the capability.
        provider: String,
    },

    /// An error returned by a provider.
    #[error("Provider `{provider}` error (status {status:?}): {message}")]
    ProviderError {
        /// Provider that returned the error.
        provider: String,
        /// HTTP status code, if applicable.
        status: Option<u16>,
        /// Error message from the provider.
        message: String,
    },

    /// The target platform is unavailable.
    #[error("Platform `{platform}` is unavailable")]
    PlatformUnavailable {
        /// Name of the unavailable platform.
        platform: String,
    },

    /// The requested model is unavailable.
    #[error("Model `{model}` is unavailable")]
    ModelUnavailable {
        /// ID of the unavailable model.
        model: String,
    },

    /// Authentication failed.
    #[error("Authentication error: {message}")]
    AuthError {
        /// Description of the authentication failure.
        message: String,
    },

    /// The request was rate limited.
    #[error("Rate limited (retry after {retry_after:?})")]
    RateLimit {
        /// Suggested wait duration before retrying.
        retry_after: Option<Duration>,
    },

    /// The request timed out.
    #[error("Timeout")]
    Timeout,

    /// The request was cancelled.
    #[error("Request was cancelled")]
    Cancelled,

    /// A network or transport error occurred.
    #[error("Transport error: {message}")]
    Transport {
        /// Error message.
        message: String,
        /// Underlying error source.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// JSON serialization or deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// A tool execution failed.
    #[error("Tool `{tool_name}` error: {message}")]
    ToolError {
        /// Name of the tool that failed.
        tool_name: String,
        /// Error message.
        message: String,
    },

    /// An internal bridge error occurred.
    #[error("Bridge `{bridge}` error: {message}")]
    BridgeError {
        /// Name of the bridge.
        bridge: String,
        /// Error message.
        message: String,
    },

    /// Output did not match the expected schema.
    #[error("Schema validation error: {message}")]
    SchemaValidation {
        /// Description of the validation failure.
        message: String,
    },

    /// An error occurred while processing a stream.
    #[error("Stream error: {message}")]
    StreamError {
        /// Error message.
        message: String,
    },

    /// The maximum number of agent steps was exceeded.
    #[error("Exceeded maximum steps ({max_steps})")]
    MaxStepsExceeded {
        /// The step limit that was reached.
        max_steps: usize,
    },
}

impl From<serde_json::Error> for AiError {
    fn from(err: serde_json::Error) -> Self {
        AiError::Serialization(err.to_string())
    }
}

/// Convenience result type alias.
pub type AiResult<T> = Result<T, AiError>;
