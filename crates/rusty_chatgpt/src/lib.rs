//! OpenAI ChatGPT provider for the Rusty AI SDK.
//!
//! This is a thin wrapper around [`rusty_openai_compatible`] that pre-configures
//! the adapter for the official OpenAI API with well-known ChatGPT models.
//!
//! # Example
//!
//! ```rust,no_run
//! use rusty_chatgpt::ChatGptProvider;
//! use rusty_ai::Provider;
//!
//! let provider = ChatGptProvider::new("sk-...");
//! let model = provider.language_model("gpt-4o").unwrap();
//! ```

use rusty_ai::capability::{Capability, CapabilitySet};
use rusty_ai::error::AiResult;
use rusty_ai::model::{EmbeddingModel, LanguageModel};
use rusty_ai::provider::Provider;
use rusty_ai::types::ModelInfo;
use rusty_openai_compatible::{
    OpenAiCompatibleConfig, OpenAiCompatibleModel, OpenAiCompatibleProvider,
};

/// A provider pre-configured for the official OpenAI ChatGPT API.
pub struct ChatGptProvider {
    inner: OpenAiCompatibleProvider,
    config: OpenAiCompatibleConfig,
}

impl ChatGptProvider {
    /// Create a new ChatGPT provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        let config = OpenAiCompatibleConfig::openai(api_key);
        let inner = OpenAiCompatibleProvider::new(config.clone(), "chatgpt", "ChatGPT")
            .with_model_info(ModelInfo {
                id: "gpt-4o".into(),
                provider: "chatgpt".into(),
                display_name: "GPT-4o".into(),
                capabilities: CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::ImageInput)
                    .with(Capability::Streaming)
                    .with(Capability::ToolCalling)
                    .with(Capability::StructuredOutput),
            })
            .with_model_info(ModelInfo {
                id: "gpt-4o-mini".into(),
                provider: "chatgpt".into(),
                display_name: "GPT-4o Mini".into(),
                capabilities: CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::ImageInput)
                    .with(Capability::Streaming)
                    .with(Capability::ToolCalling)
                    .with(Capability::StructuredOutput),
            })
            .with_model_info(ModelInfo {
                id: "o3-mini".into(),
                provider: "chatgpt".into(),
                display_name: "o3-mini".into(),
                capabilities: CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::Streaming)
                    .with(Capability::ToolCalling),
            });
        Self { inner, config }
    }

    /// Set an optional OpenAI organization ID.
    pub fn with_org(mut self, org_id: impl Into<String>) -> Self {
        self.config = self.config.with_org(org_id);
        // Rebuild inner provider with the updated config so that models
        // created via the Provider trait pick up the org header.
        let models: Vec<ModelInfo> = self.inner.models().to_vec();
        let mut new_inner =
            OpenAiCompatibleProvider::new(self.config.clone(), "chatgpt", "ChatGPT");
        for info in models {
            new_inner = new_inner.with_model_info(info);
        }
        self.inner = new_inner;
        self
    }

    /// Get a specific model by ID, looking up known capabilities.
    pub fn model(&self, model_id: &str) -> OpenAiCompatibleModel {
        let caps = self
            .inner
            .models()
            .iter()
            .find(|m| m.id == model_id)
            .map(|m| m.capabilities.clone())
            .unwrap_or_else(|| {
                CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::Streaming)
            });
        OpenAiCompatibleModel::new(self.config.clone(), model_id, "chatgpt")
            .with_capabilities(caps)
    }

    /// Convenience: get a GPT-4o model handle.
    pub fn gpt4o(&self) -> OpenAiCompatibleModel {
        self.model("gpt-4o")
    }

    /// Convenience: get a GPT-4o Mini model handle.
    pub fn gpt4o_mini(&self) -> OpenAiCompatibleModel {
        self.model("gpt-4o-mini")
    }
}

impl Provider for ChatGptProvider {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Ok(Box::new(self.model(model_id)))
    }

    fn embedding_model(&self, _model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(rusty_ai::AiError::ModelUnavailable {
            model: "ChatGPT does not support embedding models".into(),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        self.inner.models().to_vec()
    }
}
