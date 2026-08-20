use std::collections::HashMap;

use async_trait::async_trait;

use rs_ai_core::error::{AiError, AiResult};
use rs_ai_core::model::{EmbeddingModel, LanguageModel};
use rs_ai_core::provider::Provider;
use rs_ai_core::types::ModelInfo;

use crate::mock_model::{MockEmbeddingModel, MockLanguageModel};

/// A mock provider for testing that holds pre-configured mock models.
pub struct MockProvider {
    id: String,
    name: String,
    models: HashMap<String, MockLanguageModel>,
    embedding_models: HashMap<String, MockEmbeddingModel>,
}

impl MockProvider {
    /// Create a new, empty mock provider.
    pub fn new() -> Self {
        Self {
            id: "mock".to_owned(),
            name: "Mock Provider".to_owned(),
            models: HashMap::new(),
            embedding_models: HashMap::new(),
        }
    }

    /// Register a language model (builder style).
    pub fn with_model(mut self, model: MockLanguageModel) -> Self {
        self.models.insert(model.id().to_owned(), model);
        self
    }

    /// Register an embedding model (builder style).
    pub fn with_embedding_model(mut self, model: MockEmbeddingModel) -> Self {
        self.embedding_models.insert(model.id().to_owned(), model);
        self
    }

    /// Override the provider id (builder style).
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_owned();
        self
    }

    /// Override the provider display name (builder style).
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        // Since MockLanguageModel is not Clone we cannot hand out the stored
        // instance directly. Instead we create a new MockLanguageModel that
        // shares the same internal Arc state. To achieve this we would need
        // interior sharing. For simplicity, return an error if the model is
        // not registered -- callers should use `MockLanguageModel` directly
        // in most tests.
        self.models
            .get(model_id)
            .map(|model| Box::new(model.clone()) as Box<dyn LanguageModel>)
            .ok_or_else(|| AiError::ModelUnavailable {
                model: model_id.to_owned(),
            })
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        self.embedding_models
            .get(model_id)
            .map(|model| Box::new(model.clone()) as Box<dyn EmbeddingModel>)
            .ok_or_else(|| AiError::ModelUnavailable {
                model: model_id.to_owned(),
            })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        let mut infos: Vec<ModelInfo> = Vec::new();
        for model in self.models.values() {
            infos.push(ModelInfo {
                id: model.id().to_owned(),
                provider: model.provider().to_owned(),
                display_name: format!("Mock {}", model.id()),
                capabilities: rs_ai_core::CapabilitySet::new()
                    .with(rs_ai_core::Capability::TextInput)
                    .with(rs_ai_core::Capability::TextOutput),
                ..Default::default()
            });
        }
        for model in self.embedding_models.values() {
            infos.push(ModelInfo {
                id: model.id().to_owned(),
                provider: model.provider().to_owned(),
                display_name: format!("Mock Embedding {}", model.id()),
                capabilities: rs_ai_core::CapabilitySet::new()
                    .with(rs_ai_core::Capability::Embeddings),
                ..Default::default()
            });
        }
        infos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs_ai_core::registry::ProviderRegistry;

    #[test]
    fn language_model_returns_registered_model() {
        let provider = MockProvider::new()
            .with_id("openai")
            .with_model(MockLanguageModel::new("gpt-4o").with_provider("openai"));
        let model = provider.language_model("gpt-4o").unwrap();
        assert_eq!(model.model_id(), "gpt-4o");
        assert_eq!(model.provider_id(), "openai");
    }

    #[test]
    fn language_model_rejects_unknown_id() {
        let provider = MockProvider::new().with_model(MockLanguageModel::new("gpt-4o"));
        assert!(matches!(
            provider.language_model("missing"),
            Err(AiError::ModelUnavailable { model }) if model == "missing"
        ));
    }

    #[test]
    fn embedding_model_returns_registered_model() {
        let provider = MockProvider::new()
            .with_embedding_model(MockEmbeddingModel::new("text-embedding-3-small", 3));
        let model = provider.embedding_model("text-embedding-3-small").unwrap();
        assert_eq!(model.model_id(), "text-embedding-3-small");
        assert_eq!(model.dimensions(), Some(3));
    }

    #[test]
    fn registry_keeps_slashy_model_ids_after_first_slash() {
        let registry = ProviderRegistry::new().register(
            "openrouter",
            MockProvider::new()
                .with_id("openrouter")
                .with_model(MockLanguageModel::new("openai/gpt-4o").with_provider("openrouter")),
        );
        let model = registry.model("openrouter/openai/gpt-4o").unwrap();
        assert_eq!(model.model_id(), "openai/gpt-4o");
        assert_eq!(model.provider_id(), "openrouter");
    }
}
