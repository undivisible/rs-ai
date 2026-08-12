//! Provider registry — Vercel-style `create_provider_registry()` for model ID strings.
//!
//! Allows referencing models as `"provider/model-id"` strings.

use std::collections::HashMap;

use crate::error::{AiError, AiResult};
use crate::model::LanguageModel;
use crate::provider::Provider;
use crate::types::ModelInfo;

/// A registry of AI providers that can resolve model ID strings.
/// Analogous to Vercel's `createProviderRegistry()`.
///
/// # Example
/// ```ignore
/// use rs_ai_core::{create_provider_registry, Provider};
/// use rs_ai_providers::chatgpt::ChatGptProvider;
/// use rs_ai_providers::claude::ClaudeProvider;
///
/// let registry = create_provider_registry()
///     .register("openai", ChatGptProvider::new("sk-..."))
///     .register("anthropic", ClaudeProvider::new("sk-ant-..."));
///
/// let model = registry.model("openai/gpt-4o")?;
/// ```
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider under the given name.
    pub fn register(mut self, name: &str, provider: impl Provider + 'static) -> Self {
        self.providers.insert(name.to_string(), Box::new(provider));
        self
    }

    /// Resolve a model ID string in the form `"provider/model-id"`.
    ///
    /// Returns the language model if found, or an error if the provider
    /// prefix or model ID is unknown.
    pub fn model(&self, id: &str) -> AiResult<Box<dyn LanguageModel>> {
        let (provider_name, model_id) =
            id.split_once('/').ok_or_else(|| AiError::InvalidPrompt {
                message: format!("Invalid model ID format '{id}'. Expected 'provider/model-id'"),
            })?;

        let provider =
            self.providers
                .get(provider_name)
                .ok_or_else(|| AiError::ModelUnavailable {
                    model: format!(
                        "Unknown provider '{provider_name}'. Available: {}",
                        self.providers
                            .keys()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                })?;

        provider.language_model(model_id)
    }

    /// List all registered models across all providers.
    pub fn available_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();
        for provider in self.providers.values() {
            models.extend(provider.available_models());
        }
        models
    }

    /// Get a reference to a registered provider by name.
    pub fn get_provider(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Fetch the latest model metadata from every registered provider.
    ///
    /// Providers that do not implement remote discovery return their local
    /// snapshot through the trait's default implementation. Consumers can
    /// feed the result into [`crate::ModelRegistry::replace`] to obtain additions,
    /// updates, and removals.
    pub async fn fetch_models(&self) -> AiResult<Vec<ModelInfo>> {
        let mut models = Vec::new();
        for provider in self.providers.values() {
            models.extend(provider.fetch_models().await?);
        }
        Ok(models)
    }

    /// Names of all providers currently registered in this registry.
    pub fn provider_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.providers.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new empty provider registry.
pub fn create_provider_registry() -> ProviderRegistry {
    ProviderRegistry::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilitySet;
    use crate::error::AiResult;
    use crate::model::{EmbeddingModel, LanguageModel};
    use async_trait::async_trait;

    struct MockProvider {
        id: String,
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.id
        }
        fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
            Err(crate::error::AiError::ModelUnavailable {
                model: model_id.into(),
            })
        }
        fn embedding_model(&self, _mid: &str) -> AiResult<Box<dyn EmbeddingModel>> {
            Err(crate::error::AiError::UnsupportedCapability {
                capability: "embedding".into(),
                provider: self.id.clone(),
            })
        }
        fn available_models(&self) -> Vec<ModelInfo> {
            vec![ModelInfo {
                id: format!("{}/m1", self.id),
                provider: self.id.clone(),
                display_name: "M1".into(),
                capabilities: CapabilitySet::new(),
                ..Default::default()
            }]
        }
    }

    #[test]
    fn test_registry_new_empty() {
        let registry = ProviderRegistry::new();
        assert!(registry.available_models().is_empty());
    }

    #[test]
    fn test_registry_register() {
        let registry = ProviderRegistry::new().register("test", MockProvider { id: "test".into() });
        assert_eq!(registry.available_models().len(), 1);
        assert_eq!(registry.available_models()[0].id, "test/m1");
    }

    #[test]
    fn test_registry_model_invalid_format() {
        let registry = ProviderRegistry::new();
        match registry.model("invalid") {
            Err(e) => assert!(
                e.to_string().contains("Invalid model ID"),
                "expected Invalid model ID error, got: {}",
                e
            ),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn test_registry_get_provider() {
        let registry = ProviderRegistry::new().register("test", MockProvider { id: "test".into() });
        assert!(registry.get_provider("test").is_some());
        assert!(registry.get_provider("missing").is_none());
    }
}
