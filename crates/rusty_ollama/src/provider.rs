use rusty_ai::{AiError, AiResult, ModelInfo, Provider};

use crate::api_types::OllamaListResponse;
use crate::model::OllamaModel;

/// A provider handle for a local Ollama server.
///
/// Use this to discover available models and create `OllamaModel` instances.
pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create a provider pointing at the default Ollama address
    /// (`http://localhost:11434`).
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Override the base URL.
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }

    /// Create an `OllamaModel` for the given model identifier (e.g.
    /// `"llama3"`, `"mistral"`, `"nomic-embed-text"`).
    pub fn model(&self, id: &str) -> OllamaModel {
        OllamaModel::new(id).with_base_url(&self.base_url)
    }

    /// List models that are currently available on the Ollama server.
    pub async fn list_models(&self) -> AiResult<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "ollama".to_string(),
                status: Some(status.as_u16()),
                message: body,
            });
        }

        let list: OllamaListResponse =
            resp.json().await.map_err(|e| AiError::Serialization(e.to_string()))?;

        Ok(list.models.into_iter().map(|m| m.name).collect())
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn rusty_ai::LanguageModel>> {
        Ok(Box::new(self.model(model_id)))
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn rusty_ai::EmbeddingModel>> {
        Ok(Box::new(self.model(model_id)))
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        // Ollama's model list requires an async call; for the sync trait we
        // return an empty list.  Use `list_models()` for the async version.
        Vec::new()
    }
}
