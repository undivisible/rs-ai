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

// ── Well-known model identifiers ──

pub const GPT_4O: &str = "gpt-4o";
pub const GPT_4O_MINI: &str = "gpt-4o-mini";
pub const O3_MINI: &str = "o3-mini";
pub const GPT_5_4: &str = "gpt-5.4";
pub const GPT_5_4_MINI: &str = "gpt-5.4-mini";
pub const GPT_5_4_NANO: &str = "gpt-5.4-nano";

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
                id: GPT_4O.into(),
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
                id: GPT_4O_MINI.into(),
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
                id: O3_MINI.into(),
                provider: "chatgpt".into(),
                display_name: "o3-mini".into(),
                capabilities: CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::Streaming)
                    .with(Capability::ToolCalling),
            })
            .with_model_info(ModelInfo {
                id: GPT_5_4.into(),
                provider: "chatgpt".into(),
                display_name: "GPT-5.4".into(),
                capabilities: CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::ImageInput)
                    .with(Capability::Streaming)
                    .with(Capability::ToolCalling)
                    .with(Capability::StructuredOutput)
                    .with(Capability::ExtendedThinking),
            })
            .with_model_info(ModelInfo {
                id: GPT_5_4_MINI.into(),
                provider: "chatgpt".into(),
                display_name: "GPT-5.4 Mini".into(),
                capabilities: CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::ImageInput)
                    .with(Capability::Streaming)
                    .with(Capability::ToolCalling)
                    .with(Capability::StructuredOutput),
            })
            .with_model_info(ModelInfo {
                id: GPT_5_4_NANO.into(),
                provider: "chatgpt".into(),
                display_name: "GPT-5.4 Nano".into(),
                capabilities: CapabilitySet::new()
                    .with(Capability::TextInput)
                    .with(Capability::TextOutput)
                    .with(Capability::Streaming),
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

    pub fn gpt4o(&self) -> OpenAiCompatibleModel {
        self.model(GPT_4O)
    }

    pub fn gpt4o_mini(&self) -> OpenAiCompatibleModel {
        self.model(GPT_4O_MINI)
    }

    pub fn gpt54(&self) -> OpenAiCompatibleModel {
        self.model(GPT_5_4)
    }

    pub fn gpt54_mini(&self) -> OpenAiCompatibleModel {
        self.model(GPT_5_4_MINI)
    }

    pub fn gpt54_nano(&self) -> OpenAiCompatibleModel {
        self.model(GPT_5_4_NANO)
    }

    /// Fetch the list of models from the OpenAI API.
    pub async fn list_remote_models(&self) -> rusty_ai::AiResult<Vec<String>> {
        use secrecy::ExposeSecret;
        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.openai.com/v1/models")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key().expose_secret()),
            )
            .send()
            .await
            .map_err(|e| rusty_ai::AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(rusty_ai::AiError::ProviderError {
                provider: "chatgpt".into(),
                status: None,
                message: body,
            });
        }

        #[derive(serde::Deserialize)]
        struct ListModelsResponse {
            data: Vec<ModelEntry>,
        }
        #[derive(serde::Deserialize)]
        struct ModelEntry {
            id: String,
        }

        let list: ListModelsResponse = resp
            .json()
            .await
            .map_err(|e| rusty_ai::AiError::Serialization(e.to_string()))?;

        Ok(list.data.into_iter().map(|m| m.id).collect())
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
