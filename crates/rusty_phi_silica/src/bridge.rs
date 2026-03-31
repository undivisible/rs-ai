use async_trait::async_trait;

use crate::types::PhiSilicaAvailability;

/// Trait that must be implemented by the host application to bridge
/// to the Windows Phi Silica runtime.
#[async_trait]
pub trait PhiSilicaBridge: Send + Sync {
    /// Check Phi Silica availability on this device.
    async fn availability(&self) -> PhiSilicaAvailability;

    /// Generate text from a prompt.
    async fn generate(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String, String>;

    /// Stream generated text in chunks.
    ///
    /// The Windows App SDK exposes `GenerateResponseWithUpdatesAsync` which
    /// yields partial text results. This method should call that and return
    /// each partial text chunk.
    ///
    /// Default implementation falls back to calling `generate()` and returning
    /// a single-element Vec.
    async fn stream_tokens(
        &self,
        prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<Vec<String>, String> {
        // Default: call generate and return as single chunk
        let result = self.generate(prompt, max_tokens).await?;
        Ok(vec![result])
    }
}
