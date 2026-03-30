use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use rusty_ai::capability::{Capability, CapabilitySet};
use rusty_ai::error::{AiError, AiResult};
use rusty_ai::model::{EmbeddingModel, LanguageModel};
use rusty_ai::provider::Provider;
use rusty_ai::types::ModelInfo;

use crate::model::ClaudeModel;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Provider for Anthropic Claude models.
pub struct ClaudeProvider {
    api_key: SecretString,
    base_url: String,
}

impl ClaudeProvider {
    /// Create a new `ClaudeProvider` with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::from(api_key.into()),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Override the base URL (useful for proxies or testing).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Get the Claude Sonnet model.
    pub fn claude_sonnet(&self) -> ClaudeModel {
        self.model("claude-sonnet-4-20250514")
    }

    /// Get the Claude Opus model.
    pub fn claude_opus(&self) -> ClaudeModel {
        self.model("claude-opus-4-20250514")
    }

    /// Get the Claude Haiku model.
    pub fn claude_haiku(&self) -> ClaudeModel {
        self.model("claude-haiku-4-20250514")
    }

    /// Get a model by identifier.
    pub fn model(&self, model_id: &str) -> ClaudeModel {
        ClaudeModel::new(self.api_key.expose_secret(), model_id)
            .with_base_url(self.base_url.clone())
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Ok(Box::new(self.model(model_id)))
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        // Anthropic does not offer embedding models.
        Err(AiError::ModelUnavailable {
            model: model_id.to_string(),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        let caps = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::ImageInput)
            .with(Capability::Streaming)
            .with(Capability::ToolCalling);

        vec![
            ModelInfo {
                id: "claude-opus-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                display_name: "Claude Opus 4".to_string(),
                capabilities: caps.clone(),
            },
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                display_name: "Claude Sonnet 4".to_string(),
                capabilities: caps.clone(),
            },
            ModelInfo {
                id: "claude-haiku-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                display_name: "Claude Haiku 4".to_string(),
                capabilities: caps,
            },
        ]
    }
}
