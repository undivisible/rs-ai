use std::collections::HashMap;

use async_trait::async_trait;

use rusty_ai::error::{AiError, AiResult};
use rusty_ai::model::{EmbeddingModel, LanguageModel};
use rusty_ai::provider::Provider;
use rusty_ai::types::ModelInfo;

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
        Err(AiError::ModelUnavailable {
            model: format!(
                "MockProvider does not support runtime model lookup; \
                 use MockLanguageModel directly. Requested: {model_id}"
            ),
        })
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(AiError::ModelUnavailable {
            model: format!(
                "MockProvider does not support runtime model lookup; \
                 use MockEmbeddingModel directly. Requested: {model_id}"
            ),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        let mut infos: Vec<ModelInfo> = Vec::new();
        for model in self.models.values() {
            infos.push(ModelInfo {
                id: model.id().to_owned(),
                provider: model.provider().to_owned(),
                display_name: format!("Mock {}", model.id()),
                capabilities: rusty_ai::CapabilitySet::new()
                    .with(rusty_ai::Capability::TextInput)
                    .with(rusty_ai::Capability::TextOutput),
            });
        }
        for model in self.embedding_models.values() {
            infos.push(ModelInfo {
                id: model.id().to_owned(),
                provider: model.provider().to_owned(),
                display_name: format!("Mock Embedding {}", model.id()),
                capabilities: rusty_ai::CapabilitySet::new().with(rusty_ai::Capability::Embeddings),
            });
        }
        infos
    }
}
