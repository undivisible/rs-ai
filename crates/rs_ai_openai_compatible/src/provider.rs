use rs_ai_traits::{CapabilitySet, LanguageModel, ModelInfo};

use crate::config::OpenAiCompatibleConfig;
use crate::model::OpenAiCompatibleModel;

/// A provider backed by an OpenAI-compatible API.
///
/// This is not a trait impl — it is a concrete registry that knows how to
/// create [`OpenAiCompatibleModel`] instances for its registered models.
pub struct OpenAiCompatibleProvider {
    config: OpenAiCompatibleConfig,
    provider_id: String,
    provider_name: String,
    models: Vec<ModelInfo>,
}

impl OpenAiCompatibleProvider {
    /// Create a new provider.
    pub fn new(config: OpenAiCompatibleConfig, id: &str, name: &str) -> Self {
        Self {
            config,
            provider_id: id.to_string(),
            provider_name: name.to_string(),
            models: Vec::new(),
        }
    }

    /// Register a model's metadata. Returns `self` for chaining.
    pub fn with_model_info(mut self, info: ModelInfo) -> Self {
        self.models.push(info);
        self
    }

    /// Return the provider identifier.
    pub fn id(&self) -> &str {
        &self.provider_id
    }

    /// Return the human-readable provider name.
    pub fn name(&self) -> &str {
        &self.provider_name
    }

    /// List the registered model metadata.
    pub fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    /// Create a [`LanguageModel`] for the given model id.
    ///
    /// If the model id matches registered metadata, the capabilities from that
    /// metadata are attached. Otherwise a model with an empty capability set is
    /// returned (OpenAI-compatible APIs typically accept any model string).
    pub fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        let caps = self
            .models
            .iter()
            .find(|m| m.id == model_id)
            .map(|m| m.capabilities.clone())
            .unwrap_or_else(CapabilitySet::new);

        Box::new(
            OpenAiCompatibleModel::new(self.config.clone(), model_id, &self.provider_id)
                .with_capabilities(caps),
        )
    }
}
