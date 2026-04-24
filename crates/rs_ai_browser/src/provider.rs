use std::sync::Arc;

use rs_ai_traits::{
    AiError, AiResult, Capability, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo,
    Provider,
};

use crate::bridge::BrowserAiBridge;
use crate::capabilities::BrowserAiCapabilities;
use crate::model::BrowserAiModel;

/// Provider for browser-based built-in AI models.
pub struct BrowserAiProvider {
    bridge: Arc<dyn BrowserAiBridge>,
}

impl BrowserAiProvider {
    /// Create a new `BrowserAiProvider` wrapping the given bridge.
    pub fn new(bridge: impl BrowserAiBridge + 'static) -> Self {
        Self {
            bridge: Arc::new(bridge),
        }
    }

    /// Get the browser AI model.
    pub fn model(&self) -> BrowserAiModel {
        BrowserAiModel::new(self.bridge.clone())
    }

    /// Detect browser AI capabilities.
    pub async fn detect(&self) -> BrowserAiCapabilities {
        self.bridge.detect().await
    }
}

impl Provider for BrowserAiProvider {
    fn id(&self) -> &str {
        "browser"
    }

    fn name(&self) -> &str {
        "Browser AI"
    }

    fn language_model(&self, _model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Ok(Box::new(self.model()))
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "embeddings".into(),
            provider: format!("browser/{model_id}"),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "browser-ai".into(),
            provider: "browser".into(),
            display_name: "Browser Built-in AI".into(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::LocalExecution)
                .with(Capability::PlatformNative),
        }]
    }
}
