use crate::client::XaiClient;
use crate::model::XaiModel;
use crate::XaiModelId;

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

    /// Grok 4.20 Reasoning - Best for complex reasoning tasks.
    pub fn grok_4_20_reasoning(&self) -> XaiModel {
        self.model(XaiModelId::Grok420Reasoning.as_str())
    }

    /// Grok 4 - High quality model for general use and vision.
    pub fn grok_4(&self) -> XaiModel {
        self.model(XaiModelId::Grok4.as_str())
    }
}
