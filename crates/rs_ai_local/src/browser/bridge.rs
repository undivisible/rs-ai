use async_trait::async_trait;

use super::capabilities::{BrowserAiCapabilities, BrowserAiOptions};

/// Trait for browser AI bridge.
///
/// On WASM targets, implement this via `wasm-bindgen` to call the browser's
/// built-in AI APIs (Chrome Prompt API, Edge AI, etc.).
///
/// On non-WASM targets, a no-op implementation can be used for testing.
///
/// # Browser compatibility notes
///
/// - `window.ai` is deprecated since Chrome 138; use the `LanguageModel` global directly.
/// - Edge AI is backed by Phi Silica on Copilot+ PCs.
/// - Not available in Chromium/CEF builds, only official Chrome/Edge.
#[async_trait]
pub trait BrowserAiBridge: Send + Sync {
    /// Detect if the browser has built-in AI capabilities.
    async fn detect(&self) -> BrowserAiCapabilities;

    /// Generate text using the browser's AI.
    async fn generate(&self, prompt: &str, options: &BrowserAiOptions) -> Result<String, String>;

    /// Stream text using the browser's AI (if supported).
    /// Returns chunks of text.
    async fn stream(&self, prompt: &str, options: &BrowserAiOptions)
        -> Result<Vec<String>, String>;
}

/// A no-op bridge for non-WASM targets, useful for testing.
pub struct NoOpBrowserBridge;

#[async_trait]
impl BrowserAiBridge for NoOpBrowserBridge {
    async fn detect(&self) -> BrowserAiCapabilities {
        BrowserAiCapabilities::default()
    }

    async fn generate(&self, _prompt: &str, _options: &BrowserAiOptions) -> Result<String, String> {
        Err("Browser AI not available on this target".into())
    }

    async fn stream(
        &self,
        _prompt: &str,
        _options: &BrowserAiOptions,
    ) -> Result<Vec<String>, String> {
        Err("Browser AI not available on this target".into())
    }
}
