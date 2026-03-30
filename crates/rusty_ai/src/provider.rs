use async_trait::async_trait;

use crate::error::AiResult;
use crate::model::{EmbeddingModel, LanguageModel};
use crate::types::ModelInfo;

/// A provider that exposes one or more language and/or embedding models.
#[async_trait]
pub trait Provider: Send + Sync {
    /// A unique identifier for this provider (e.g. "openai", "anthropic").
    fn id(&self) -> &str;

    /// A human-readable display name.
    fn name(&self) -> &str;

    /// Retrieve a language model by its identifier.
    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>>;

    /// Retrieve an embedding model by its identifier.
    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>>;

    /// List the models available from this provider.
    fn available_models(&self) -> Vec<ModelInfo>;
}
