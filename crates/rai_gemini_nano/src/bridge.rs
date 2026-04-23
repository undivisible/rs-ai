use async_trait::async_trait;

use crate::types::{ModelDownloadState, NanoCapabilities, NanoSessionConfig};

/// Trait that must be implemented by the host application to bridge
/// to the Android Prompt API via JNI/Kotlin interop.
///
/// The host app provides a concrete implementation that calls into the
/// Android platform SDK. This crate consumes that implementation to
/// expose a standard [`rai_ai::LanguageModel`].
#[async_trait]
pub trait GeminiNanoBridge: Send + Sync {
    /// Check if Gemini Nano is available on this device.
    async fn is_available(&self) -> bool;

    /// Get the current download state of the model.
    async fn download_state(&self) -> ModelDownloadState;

    /// Request model download if not already downloaded.
    async fn request_download(&self) -> Result<(), String>;

    /// Get device capabilities.
    async fn capabilities(&self) -> NanoCapabilities;

    /// Generate text from a prompt (single-turn).
    async fn generate(&self, prompt: &str, config: &NanoSessionConfig) -> Result<String, String>;

    /// Create a new session for multi-turn conversation.
    /// Returns a session identifier.
    async fn create_session(&self, config: &NanoSessionConfig) -> Result<String, String>;

    /// Send a message in an existing session.
    async fn send_message(&self, session_id: &str, message: &str) -> Result<String, String>;

    /// Close/destroy a session.
    async fn close_session(&self, session_id: &str) -> Result<(), String>;
}
