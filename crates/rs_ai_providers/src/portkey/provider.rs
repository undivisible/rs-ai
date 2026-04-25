use super::client::PortkeyClient;
use super::model::PortkeyModel;

/// Portkey AI Gateway provider for creating models.
///
/// Portkey is an LLM gateway that provides multi-provider routing, load balancing,
/// automatic fallback/retry, caching, and observability features.
#[derive(Clone)]
pub struct PortkeyProvider {
    client: std::sync::Arc<PortkeyClient>,
}

impl PortkeyProvider {
    /// Create a new Portkey provider with the given API key.
    ///
    /// The API key is read from the `PORTKEY_API_KEY` environment variable if not provided.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let provider = PortkeyProvider::new("pk_...");
    /// let model = provider.model("gpt-4");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = PortkeyClient::new(api_key);
        Self {
            client: std::sync::Arc::new(client),
        }
    }

    /// Create a new Portkey provider with a custom base URL.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let provider = PortkeyProvider::new("pk_...")
    ///     .with_base_url("https://custom.portkey.ai/v1");
    /// ```
    pub fn with_base_url(self, url: impl Into<String>) -> Self {
        let api_key = self.client.get_api_key();
        let client = PortkeyClient::new(api_key).with_base_url(url);
        Self {
            client: std::sync::Arc::new(client),
        }
    }

    /// Create a model instance for the given model ID string.
    ///
    /// The model ID is the OpenAI-compatible model identifier that Portkey will route to.
    pub fn model(&self, model_id: &str) -> PortkeyModel {
        PortkeyModel::new(model_id.to_string(), self.client.clone())
    }
}
