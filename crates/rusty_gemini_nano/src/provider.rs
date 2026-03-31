use std::sync::Arc;

use rusty_ai::{
    AiError, AiResult, Capability, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo,
    Provider,
};

use crate::bridge::GeminiNanoBridge;
use crate::model::GeminiNanoModel;
use crate::session::NanoSession;
use crate::types::{ModelDownloadState, NanoSessionConfig};

/// Provider for Gemini Nano on-device inference via the Android Prompt API.
pub struct GeminiNanoProvider {
    bridge: Arc<dyn GeminiNanoBridge>,
}

impl GeminiNanoProvider {
    pub fn new(bridge: impl GeminiNanoBridge + 'static) -> Self {
        Self {
            bridge: Arc::new(bridge),
        }
    }

    /// Get the Gemini Nano language model.
    pub fn model(&self) -> GeminiNanoModel {
        GeminiNanoModel::new(self.bridge.clone())
    }

    /// Check if Gemini Nano is available on this device.
    pub async fn is_available(&self) -> bool {
        self.bridge.is_available().await
    }

    /// Get the current download state of the model.
    pub async fn download_state(&self) -> ModelDownloadState {
        self.bridge.download_state().await
    }

    /// Request model download if not already downloaded.
    pub async fn request_download(&self) -> AiResult<()> {
        self.bridge
            .request_download()
            .await
            .map_err(|e| AiError::BridgeError {
                bridge: "gemini_nano".into(),
                message: e,
            })
    }

    /// Create a new multi-turn session.
    pub async fn create_session(&self, config: NanoSessionConfig) -> AiResult<NanoSession> {
        let session_id =
            self.bridge
                .create_session(&config)
                .await
                .map_err(|e| AiError::BridgeError {
                    bridge: "gemini_nano".into(),
                    message: e,
                })?;
        Ok(NanoSession::new(session_id, self.bridge.clone(), config))
    }
}

impl Provider for GeminiNanoProvider {
    fn id(&self) -> &str {
        "gemini_nano"
    }

    fn name(&self) -> &str {
        "Gemini Nano (Android)"
    }

    fn language_model(&self, _model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Ok(Box::new(self.model()))
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "embeddings".into(),
            provider: format!("gemini_nano/{model_id}"),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: "gemini-nano".into(),
            provider: "gemini_nano".into(),
            display_name: "Gemini Nano (On-Device)".into(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::LocalExecution)
                .with(Capability::SessionSupport)
                .with(Capability::PlatformNative),
        }]
    }
}
