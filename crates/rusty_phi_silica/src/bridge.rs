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
}
