use super::client::XaiClient;
use super::image::XaiImageModel;
use super::model::XaiModel;
use super::realtime::{create_grok_voice_session, GrokVoiceConfig, GrokVoiceSession};
use super::speech::XaiSpeechModel;
use super::transcription::XaiTranscriptionModel;
use super::video::XaiVideoModel;
use crate::XaiModelId;
use rs_ai_core::{AiResult, CacheConfig};

/// xAI provider for creating Grok models.
#[derive(Clone)]
pub struct XaiProvider {
    client: std::sync::Arc<XaiClient>,
    oauth_token: Option<String>,
}

impl XaiProvider {
    /// Create a new xAI provider with the given API key.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let provider = XaiProvider::new("your-api-key");
    /// let model = provider.grok_4_20_reasoning();
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = XaiClient::new(api_key);
        Self {
            client: std::sync::Arc::new(client),
            oauth_token: None,
        }
    }

    /// Override the API key with an OAuth bearer token.
    pub fn with_oauth_token(mut self, token: impl Into<String>) -> Self {
        self.oauth_token = Some(token.into());
        self
    }

    fn effective_client(&self) -> std::sync::Arc<XaiClient> {
        if let Some(ref token) = self.oauth_token {
            std::sync::Arc::new(XaiClient::new(token.as_str()))
        } else {
            self.client.clone()
        }
    }

    /// Create a model instance for the given model ID.
    pub fn model(&self, model_id: &str) -> XaiModel {
        XaiModel::new(model_id.to_string(), self.effective_client())
    }

    /// Create a model instance with cache configuration.
    pub fn model_with_cache(&self, model_id: &str, cache_config: CacheConfig) -> XaiModel {
        let mut model = XaiModel::new(model_id.to_string(), self.effective_client());
        model.set_cache(cache_config);
        model
    }

    /// Grok 4.20 Reasoning - Best for complex reasoning tasks.
    pub fn grok_4_20_reasoning(&self) -> XaiModel {
        self.model(XaiModelId::Grok420Reasoning.as_str())
    }

    /// Grok 4 - High quality model for general use and vision.
    pub fn grok_4(&self) -> XaiModel {
        self.model(XaiModelId::Grok4.as_str())
    }

    /// Grok 4.20 Reasoning with cache configuration.
    pub fn grok_4_20_reasoning_with_cache(&self, cache_config: CacheConfig) -> XaiModel {
        self.model_with_cache(XaiModelId::Grok420Reasoning.as_str(), cache_config)
    }

    /// Grok 4 with cache configuration.
    pub fn grok_4_with_cache(&self, cache_config: CacheConfig) -> XaiModel {
        self.model_with_cache(XaiModelId::Grok4.as_str(), cache_config)
    }

    /// Create an image generation model for the given model ID.
    pub fn image_model(&self, model_id: &str) -> XaiImageModel {
        XaiImageModel::new(model_id.to_string(), self.effective_client())
    }

    /// Grok 4 Imagine — xAI's image generation model.
    pub fn grok_4_imagine(&self) -> XaiImageModel {
        self.image_model(XaiModelId::Grok4Imagine.as_str())
    }

    /// Aurora — xAI's image generation model.
    pub fn aurora(&self) -> XaiImageModel {
        self.image_model(XaiModelId::Aurora.as_str())
    }

    /// Create a text-to-speech model.
    pub fn speech_model(&self, model_id: &str) -> XaiSpeechModel {
        XaiSpeechModel::new(
            self.effective_client().api_key().to_string(),
            model_id.to_string(),
        )
    }

    /// Create a speech-to-text model.
    pub fn stt_model(&self, model_id: &str) -> XaiTranscriptionModel {
        XaiTranscriptionModel::new(
            self.effective_client().api_key().to_string(),
            model_id.to_string(),
        )
    }

    /// Create a video generation model.
    pub fn video_model(&self, model_id: &str) -> XaiVideoModel {
        XaiVideoModel::new(
            self.effective_client().api_key().to_string(),
            model_id.to_string(),
        )
    }

    /// Create a Grok Voice Agent session.
    ///
    /// Connects to xAI's Realtime API via WebSocket.
    pub async fn realtime_session(&self, model_id: &str) -> AiResult<GrokVoiceSession> {
        let api_key = self
            .oauth_token
            .clone()
            .unwrap_or_else(|| self.client.api_key().to_string());
        let config = GrokVoiceConfig::new(api_key).with_model(model_id);
        create_grok_voice_session(config).await
    }
}
