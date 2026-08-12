use std::sync::Arc;

use rs_ai_core::{
    AiError, AiResult, Capability, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo,
    Provider,
};

use super::bridge::PhiSilicaBridge;
use super::model::PhiSilicaModel;
use super::types::PhiSilicaAvailability;

/// Provider for Windows Phi Silica on-device inference.
pub struct PhiSilicaProvider {
    bridge: Arc<dyn PhiSilicaBridge>,
}

impl PhiSilicaProvider {
    /// Create a new `PhiSilicaProvider` wrapping the given bridge.
    pub fn new(bridge: impl PhiSilicaBridge + 'static) -> Self {
        Self {
            bridge: Arc::new(bridge),
        }
    }

    /// Try to create a provider using the native C# bridge.
    ///
    /// Returns `None` if the C# bridge is not available (e.g., not on Windows,
    /// or the Windows App SDK is not installed).
    pub fn try_native() -> Option<Self> {
        if super::native_bridge::native_bridge_available() {
            Some(Self::new(super::native_bridge::NativePhiSilicaBridge))
        } else {
            None
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
            ..Default::default()
        }]
    }
}
