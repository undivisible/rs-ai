//! Bridge trait for Gemini Nano via the ML Kit GenAI Prompt API.
//!
//! On Android, use [`JniGeminiNanoBridge`](crate::JniGeminiNanoBridge) which
//! calls `GenAI.getClient()` via JNI.

use async_trait::async_trait;

use super::types::{NanoContentPart, NanoGenerateResult, NanoGenerationConfig};

#[allow(deprecated)]
use super::types::{ModelDownloadState, NanoCapabilities, NanoSessionConfig};

/// Trait that must be implemented by the host application to bridge
/// to the ML Kit GenAI Prompt API (Gemini Nano on-device).
///
/// On Android targets, the `JniGeminiNanoBridge` type implements
/// this trait via JNI. On non-Android targets, use `MockNanoBridge` for testing.
///
/// # New API (preferred)
///
/// - `is_available` — check if AICore is ready
/// - `generate_content` — generate from text + optional images
///
/// # Deprecated
///
/// Methods like `download_state`, `request_download`, `capabilities`,
/// `create_session`, `send_message`, `close_session` are kept with default
/// implementations for backward compatibility but are deprecated.
#[async_trait]
pub trait GeminiNanoBridge: Send + Sync {
    // ─── New API ──────────────────────────────────────────────────────────

    /// Check if Gemini Nano is available on this device (AICore ready).
    async fn is_available(&self) -> bool;

    /// Generate content from text and optional image parts.
    async fn generate_content(
        &self,
        parts: Vec<NanoContentPart>,
        config: &NanoGenerationConfig,
    ) -> Result<NanoGenerateResult, String>;

    // ─── Deprecated backward-compat methods ────────────────────────────────

    /// Get the current download state of the model.
    ///
    /// ⚠️ Deprecated — AICore manages downloads automatically.
    #[deprecated(
        since = "0.3.0",
        note = "AICore handles downloads; use is_available() to check readiness"
    )]
    #[allow(deprecated)]
    async fn download_state(&self) -> ModelDownloadState {
        ModelDownloadState::Downloaded
    }

    /// Request model download if not already downloaded.
    ///
    /// ⚠️ Deprecated — AICore manages downloads automatically.
    #[deprecated(since = "0.3.0", note = "AICore handles downloads automatically")]
    async fn request_download(&self) -> Result<(), String> {
        Ok(())
    }

    /// Get device capabilities.
    ///
    /// ⚠️ Deprecated — use `is_available()` instead.
    #[deprecated(since = "0.3.0", note = "Use is_available() instead")]
    #[allow(deprecated)]
    async fn capabilities(&self) -> NanoCapabilities {
        NanoCapabilities {
            text_generation: true,
            summarization: false,
            rewriting: false,
        }
    }

    /// Generate text from a prompt (single-turn).
    ///
    /// ⚠️ Deprecated — use `generate_content` instead.
    #[deprecated(
        since = "0.3.0",
        note = "Use generate_content with NanoContentPart::Text"
    )]
    #[allow(deprecated)]
    async fn generate(&self, prompt: &str, config: &NanoSessionConfig) -> Result<String, String> {
        let parts = vec![NanoContentPart::Text(prompt.to_string())];
        let gen_config = NanoGenerationConfig {
            temperature: config.temperature.map(|t| t as f32),
            max_output_tokens: config.max_tokens,
            ..Default::default()
        };
        self.generate_content(parts, &gen_config)
            .await
            .map(|r| r.text)
    }

    /// Create a new session for multi-turn conversation.
    ///
    /// ⚠️ Deprecated — sessions are not supported by the new ML Kit API.
    #[deprecated(
        since = "0.3.0",
        note = "Sessions not supported by ML Kit GenAI Prompt API"
    )]
    #[allow(deprecated)]
    async fn create_session(&self, _config: &NanoSessionConfig) -> Result<String, String> {
        Err("create_session is deprecated. ML Kit GenAI Prompt API does not support sessions. Use the generate_content method instead.".into())
    }

    /// Send a message in an existing session.
    ///
    /// ⚠️ Deprecated — sessions are not supported by the new ML Kit API.
    #[deprecated(
        since = "0.3.0",
        note = "Sessions not supported by ML Kit GenAI Prompt API"
    )]
    async fn send_message(&self, _session_id: &str, _message: &str) -> Result<String, String> {
        Err("send_message is deprecated. ML Kit GenAI Prompt API does not support sessions. Use generate_content instead.".into())
    }

    /// Close/destroy a session.
    ///
    /// ⚠️ Deprecated — sessions are not supported by the new ML Kit API.
    #[deprecated(
        since = "0.3.0",
        note = "Sessions not supported by ML Kit GenAI Prompt API"
    )]
    async fn close_session(&self, _session_id: &str) -> Result<(), String> {
        Ok(())
    }
}

// ─── Mock bridge for non-Android targets ──────────────────────────────────

/// A mock bridge for non-Android targets, useful for testing.
pub struct MockNanoBridge;

#[async_trait]
impl GeminiNanoBridge for MockNanoBridge {
    async fn is_available(&self) -> bool {
        true
    }

    async fn generate_content(
        &self,
        parts: Vec<NanoContentPart>,
        _config: &NanoGenerationConfig,
    ) -> Result<NanoGenerateResult, String> {
        let text_parts: Vec<String> = parts
            .into_iter()
            .map(|p| match p {
                NanoContentPart::Text(t) => t,
                NanoContentPart::Image { .. } => "[image]".to_string(),
            })
            .collect();
        let joined = text_parts.join("\n");
        Ok(NanoGenerateResult {
            text: format!("[Gemini Nano mock response to: {joined}]"),
        })
    }
}

// ─── Arc<JniGeminiNanoBridge> delegation ──────────────────────────────────

/// Helper to deserialize the JSON result from the JNI bridge's
/// generate_content_json call.
#[cfg(target_os = "android")]
fn parse_result_json(json: &str) -> Result<NanoGenerateResult, String> {
    serde_json::from_str::<NanoGenerateResult>(json)
        .map_err(|e| format!("failed to parse NanoGenerateResult from JNI: {e}"))
}

/// Helper to serialize parts + config to JSON for the JNI bridge.
#[cfg(target_os = "android")]
fn serialize_request(
    parts: &[NanoContentPart],
    config: &NanoGenerationConfig,
) -> Result<String, String> {
    #[derive(serde::Serialize)]
    struct Request<'a> {
        parts: &'a [NanoContentPart],
        config: &'a NanoGenerationConfig,
    }
    serde_json::to_string(&Request { parts, config })
        .map_err(|e| format!("failed to serialize generate_content request: {e}"))
}

#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg(target_os = "android")]
#[async_trait]
impl GeminiNanoBridge for Arc<crate::JniGeminiNanoBridge> {
    async fn is_available(&self) -> bool {
        self.as_ref().is_available()
    }

    async fn generate_content(
        &self,
        parts: Vec<NanoContentPart>,
        config: &NanoGenerationConfig,
    ) -> Result<NanoGenerateResult, String> {
        let json = serialize_request(&parts, config)?;
        let result_json = self.as_ref().generate_content_json(&json)?;
        parse_result_json(&result_json)
    }
}
