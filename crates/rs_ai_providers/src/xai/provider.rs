use super::client::XaiClient;
use super::model::XaiModel;
use crate::XaiModelId;
use rs_ai_core::CacheConfig;

/// xAI provider for creating Grok models.
#[derive(Clone)]
pub struct XaiProvider {
    client: std::sync::Arc<XaiClient>,
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
        }
    }

    /// Create a model instance for the given model ID.
    pub fn model(&self, model_id: &str) -> XaiModel {
        XaiModel::new(model_id.to_string(), self.client.clone())
    }

    /// Create a model instance with cache configuration.
    pub fn model_with_cache(&self, model_id: &str, cache_config: CacheConfig) -> XaiModel {
        let mut model = XaiModel::new(model_id.to_string(), self.client.clone());
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
}
