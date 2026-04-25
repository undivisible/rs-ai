//! Provider trait.
use async_trait::async_trait;

use crate::error::{AiError, AiResult};
use crate::model::{EmbeddingModel, LanguageModel, SpeechToTextModel, TextToSpeechModel};
use crate::types::ModelInfo;

/// A provider that exposes one or more language and/or embedding models.
#[async_trait]
pub trait Provider: Send + Sync {
    /// A unique identifier for this provider (e.g. "openai", "anthropic").
    fn id(&self) -> &str;

    /// A human-readable display name.
    fn name(&self) -> &str;

    /// Retrieve a language model by its identifier.
    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>>;

    /// Retrieve an embedding model by its identifier.
    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>>;

    /// List the locally-known models for this provider.
    ///
    /// This returns a static snapshot of registered models.  For providers
    /// that can dynamically discover models (cloud APIs, Ollama), prefer
    /// [`Provider::fetch_models`] which queries the remote API.
    fn available_models(&self) -> Vec<ModelInfo>;

    /// Fetch the list of models from the remote API.
    ///
    /// Not all providers support dynamic discovery.  The default
    /// implementation falls back to [`Provider::available_models`].
    async fn fetch_models(&self) -> AiResult<Vec<ModelInfo>> {
        Ok(self.available_models())
    }

    /// Retrieve a speech-to-text model by its identifier.
    fn speech_to_text_model(&self, _model_id: &str) -> AiResult<Box<dyn SpeechToTextModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "speech_to_text".into(),
            provider: self.id().into(),
        })
    }

    /// Retrieve a text-to-speech model by its identifier.
    fn text_to_speech_model(&self, _model_id: &str) -> AiResult<Box<dyn TextToSpeechModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "text_to_speech".into(),
            provider: self.id().into(),
        })
    }
}
