use std::sync::Arc;

use rs_ai_core::{
    AiError, AiResult, Capability, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo,
    Provider,
};

use super::bridge::GeminiNanoBridge;
use super::model::GeminiNanoModel;
use super::session::NanoSession;
#[allow(deprecated)]
use super::types::NanoSessionConfig;

/// Provider for Gemini Nano on-device inference via the ML Kit GenAI Prompt API.
pub struct GeminiNanoProvider {
    bridge: Arc<dyn GeminiNanoBridge>,
}

impl GeminiNanoProvider {
    /// Create a new `GeminiNanoProvider` wrapping the given bridge.
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
    ///
    /// ⚠️ Deprecated — AICore manages downloads automatically.
    #[allow(deprecated)]
    pub async fn download_state(&self) -> super::types::ModelDownloadState {
        self.bridge.download_state().await
    }

    /// Request model download if not already downloaded.
    ///
    /// ⚠️ Deprecated — AICore manages downloads automatically.
    #[allow(deprecated)]
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
    ///
    /// ⚠️ Deprecated — The ML Kit GenAI Prompt API does not support sessions.
    #[allow(deprecated)]
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
                .with(Capability::ImageInput)
                .with(Capability::LocalExecution)
                .with(Capability::PlatformNative),
            ..Default::default()
        }]
    }
}
