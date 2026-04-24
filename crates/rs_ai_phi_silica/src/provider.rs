use std::sync::Arc;

use rs_ai_ai::{
    AiError, AiResult, Capability, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo,
    Provider,
};

use crate::bridge::PhiSilicaBridge;
use crate::model::PhiSilicaModel;
use crate::types::PhiSilicaAvailability;

/// Provider for Windows Phi Silica on-device inference.
pub struct PhiSilicaProvider {
    bridge: Arc<dyn PhiSilicaBridge>,
}

impl PhiSilicaProvider {
    pub fn new(bridge: impl PhiSilicaBridge + 'static) -> Self {
        Self {
            bridge: Arc::new(bridge),
        }
    }

    /// Get the Phi Silica language model.
    pub fn model(&self) -> PhiSilicaModel {
        PhiSilicaModel::new(self.bridge.clone())
    }

    /// Check Phi Silica availability.
    pub async fn availability(&self) -> PhiSilicaAvailability {
        self.bridge.availability().await
    }
}

impl Provider for PhiSilicaProvider {
    fn id(&self) -> &str {
        "phi_silica"
    }

    fn name(&self) -> &str {
        "Phi Silica (Windows)"
    }

    fn language_model(&self, _model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Ok(Box::new(self.model()))
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "embeddings".into(),
            provider: format!("phi_silica/{model_id}"),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "phi-silica".into(),
            provider: "phi_silica".into(),
            display_name: "Phi Silica (Windows NPU)".into(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::LocalExecution)
                .with(Capability::PlatformNative),
        }]
    }
}
