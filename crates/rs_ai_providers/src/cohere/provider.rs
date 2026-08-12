use crate::cohere::rerank::CohereRerankingModel;
use crate::cohere::RERANK_MODEL;
use rs_ai_core::{
    AiError, AiResult, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo, Provider,
};

/// Cohere API provider for reranking models.
pub struct CohereProvider {
    api_key: String,
    base_url: String,
}

impl CohereProvider {
    /// Create a new Cohere provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.cohere.com".into(),
        }
    }

    /// Override the base URL (e.g., for self-hosted or proxy).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Create a reranking model for the given model ID.
    pub fn reranking_model(&self, model_id: &str) -> CohereRerankingModel {
        CohereRerankingModel::new(model_id, &self.api_key, &self.base_url)
    }
}

impl Provider for CohereProvider {
    fn id(&self) -> &str {
        "cohere"
    }

    fn name(&self) -> &str {
        "Cohere"
    }

    fn language_model(&self, _model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "language_model".into(),
            provider: "cohere".into(),
        })
    }

    fn embedding_model(&self, _model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "embedding_model".into(),
            provider: "cohere".into(),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: RERANK_MODEL.to_string(),
            provider: "cohere".to_string(),
            display_name: "Cohere Rerank".to_string(),
            capabilities: CapabilitySet::default(),
            ..Default::default()
        }]
    }
}
