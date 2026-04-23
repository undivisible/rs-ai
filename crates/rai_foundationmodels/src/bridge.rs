use async_trait::async_trait;

use crate::types::{AppleModelAvailability, FoundationModelConfig};

/// Trait that must be implemented by the host application to bridge
/// to Apple's Foundation Models framework via Swift/ObjC interop.
///
/// See `undivisible/rai_foundationmodels` for the reference Swift bridge.
#[async_trait]
pub trait FoundationModelBridge: Send + Sync {
    /// Check model availability on this device.
    async fn availability(&self) -> AppleModelAvailability;

    /// Generate text from a prompt.
    async fn generate(
        &self,
        prompt: &str,
        config: &FoundationModelConfig,
    ) -> Result<String, String>;

    /// Stream text from a prompt, returning chunks.
    /// Apple Foundation Models supports streaming via AsyncSequence.
    async fn stream(
        &self,
        prompt: &str,
        config: &FoundationModelConfig,
    ) -> Result<Vec<String>, String>;
}
