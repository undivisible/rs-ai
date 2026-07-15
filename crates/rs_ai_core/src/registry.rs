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
        let (provider_name, model_id) = id.split_once('/').ok_or_else(|| {
            AiError::InvalidPrompt {
                message: format!(
                    "Invalid model ID format '{id}'. Expected 'provider/model-id'"
                ),
            }
        })?;

        let provider = self.providers.get(provider_name).ok_or_else(|| {
            AiError::ModelUnavailable {
                model: format!(
                    "Unknown provider '{provider_name}'. Available: {}",
                    self.providers.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            }
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
