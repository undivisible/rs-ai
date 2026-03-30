use std::sync::Arc;

use rusty_ai::{
    AiError, AiResult, Capability, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo,
    Provider,
};

use crate::bridge::FoundationModelBridge;
use crate::model::FoundationModel;
use crate::types::AppleModelAvailability;

/// Provider for Apple Foundation Models on-device inference.
pub struct FoundationModelProvider {
    bridge: Arc<dyn FoundationModelBridge>,
}

impl FoundationModelProvider {
    pub fn new(bridge: impl FoundationModelBridge + 'static) -> Self {
        Self {
            bridge: Arc::new(bridge),
        }
    }

    /// Get the Foundation Model language model.
    pub fn model(&self) -> FoundationModel {
        FoundationModel::new(self.bridge.clone())
    }

    /// Check model availability on this device.
    pub async fn availability(&self) -> AppleModelAvailability {
        self.bridge.availability().await
    }
}

impl Provider for FoundationModelProvider {
    fn id(&self) -> &str {
        "foundationmodels"
    }

    fn name(&self) -> &str {
        "Apple Foundation Models"
    }

    fn language_model(&self, _model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Ok(Box::new(self.model()))
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "embeddings".into(),
            provider: format!("foundationmodels/{model_id}"),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "apple-foundation-model".into(),
            provider: "foundationmodels".into(),
            display_name: "Apple Foundation Model (On-Device)".into(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::LocalExecution)
                .with(Capability::PlatformNative),
        }]
    }
}
