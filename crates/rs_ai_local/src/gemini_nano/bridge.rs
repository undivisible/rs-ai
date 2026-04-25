//! Bridge trait for Gemini Nano.
//!
//! On Android, use [`JniGeminiNanoBridge`](crate::JniGeminiNanoBridge) which
//! implements this trait by calling into Kotlin via JNI.

use async_trait::async_trait;

use super::types::{ModelDownloadState, NanoCapabilities, NanoSessionConfig};

/// Trait that must be implemented by the host application to bridge
/// to the Android Prompt API.
///
/// On Android targets, the [`crate::JniGeminiNanoBridge`] type implements
/// this trait by calling into Kotlin via JNI. On non-Android targets, you
/// must provide your own implementation or use the mock bridge for testing.
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

/// A mock bridge for non-Android targets, useful for testing.
pub struct MockNanoBridge;

#[async_trait]
impl GeminiNanoBridge for MockNanoBridge {
    async fn is_available(&self) -> bool {
        true
    }

    async fn download_state(&self) -> ModelDownloadState {
        ModelDownloadState::Downloaded
    }

    async fn request_download(&self) -> Result<(), String> {
        Ok(())
    }

    async fn capabilities(&self) -> NanoCapabilities {
        NanoCapabilities {
            text_generation: true,
            summarization: false,
            rewriting: false,
        }
    }

    async fn generate(&self, prompt: &str, _config: &NanoSessionConfig) -> Result<String, String> {
        Ok(format!("[Gemini Nano mock response to: {prompt}]"))
    }

    async fn create_session(&self, _config: &NanoSessionConfig) -> Result<String, String> {
        Ok("mock-session-1".into())
    }

    async fn send_message(&self, _session_id: &str, message: &str) -> Result<String, String> {
        Ok(format!("[Session reply to: {message}]"))
    }

    async fn close_session(&self, _session_id: &str) -> Result<(), String> {
        Ok(())
    }
}

// ─── JniGeminiNanoBridge trait implementation ──────────────────────────────────

#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg(target_os = "android")]
#[async_trait]
impl GeminiNanoBridge for Arc<crate::JniGeminiNanoBridge> {
    async fn is_available(&self) -> bool {
        crate::JniGeminiNanoBridge::is_available(self)
    }

    async fn download_state(&self) -> ModelDownloadState {
        crate::JniGeminiNanoBridge::download_state(self)
    }

    async fn request_download(&self) -> Result<(), String> {
        crate::JniGeminiNanoBridge::request_download(self)
    }

    async fn capabilities(&self) -> NanoCapabilities {
        crate::JniGeminiNanoBridge::capabilities(self)
    }

    async fn generate(&self, prompt: &str, config: &NanoSessionConfig) -> Result<String, String> {
        crate::JniGeminiNanoBridge::generate(self, prompt, config)
    }

    async fn create_session(&self, config: &NanoSessionConfig) -> Result<String, String> {
        crate::JniGeminiNanoBridge::create_session(self, config)
    }

    async fn send_message(&self, session_id: &str, message: &str) -> Result<String, String> {
        crate::JniGeminiNanoBridge::send_message(self, session_id, message)
    }

    async fn close_session(&self, session_id: &str) -> Result<(), String> {
        crate::JniGeminiNanoBridge::close_session(self, session_id)
    }
}
