//! Common types for model metadata and finish reasons.
use std::collections::BTreeSet;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capability::{Capability, CapabilitySet};

/// Reason the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Generation completed naturally.
    #[default]
    Stop,
    /// Maximum token limit reached.
    Length,
    /// Stopped because a tool call was requested.
    ToolCall,
    /// Content was filtered.
    ContentFilter,
    /// An error occurred during generation.
    Error,
    /// Finish reason is unknown or unspecified.
    Unknown,
}

/// Token and request limits advertised by a provider.
///
/// These values are optional because most OpenAI-compatible `/models`
/// endpoints only return an ID and owner.  They must never be treated as
/// universal guarantees when the provider did not publish them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelLimits {
    /// Maximum total input-plus-output context window, in tokens.
    pub context_window: Option<u64>,
    /// Maximum input tokens, when the provider publishes a separate limit.
    pub max_input_tokens: Option<u64>,
    /// Maximum generated output tokens.
    pub max_output_tokens: Option<u64>,
}

/// Price information advertised by a provider.
///
/// Values are denominated in the provider's advertised unit.  For OpenRouter
/// this is USD per token (not per million tokens), matching its models API.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Price per input token.
    pub input_per_token: Option<f64>,
    /// Price per output token.
    pub output_per_token: Option<f64>,
    /// Fixed price per request.
    pub request: Option<f64>,
    /// Price per image input.
    pub image_input: Option<f64>,
    /// Price per internal reasoning token.
    pub reasoning: Option<f64>,
    /// Price per cached input token read.
    pub cache_read: Option<f64>,
    /// Price per cached input token written.
    pub cache_write: Option<f64>,
}

/// Metadata describing a model exposed by a provider.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier.
    pub id: String,
    /// Provider identifier.
    pub provider: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Capabilities supported by this model.
    pub capabilities: CapabilitySet,
    /// Provider-advertised limits, when known.
    #[serde(default)]
    pub limits: ModelLimits,
    /// Provider-advertised pricing, when known.
    #[serde(default)]
    pub pricing: Option<ModelPricing>,
    /// Input modalities such as `text`, `image`, `audio`, or `file`.
    #[serde(default)]
    pub input_modalities: Vec<String>,
    /// Output modalities such as `text`, `image`, or `audio`.
    #[serde(default)]
    pub output_modalities: Vec<String>,
    /// Parameters explicitly supported by the provider for this model.
    #[serde(default)]
    pub supported_parameters: BTreeSet<String>,
    /// Human-readable provider description, if available.
    #[serde(default)]
    pub description: Option<String>,
    /// Provider creation timestamp, if available.
    #[serde(default)]
    pub created: Option<u64>,
    /// Provider ownership/author field, if available.
    #[serde(default)]
    pub owned_by: Option<String>,
    /// Deprecation or expiration date as supplied by the provider.
    #[serde(default)]
    pub expiration_date: Option<String>,
}

/// Metadata attached to an outgoing request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Unique request identifier.
    pub request_id: Uuid,
    /// Timestamp when the request was created.
    pub timestamp: DateTime<Utc>,
    /// Arbitrary extra metadata.
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for RequestMetadata {
    fn default() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            extra: HashMap::new(),
        }
    }
}

/// Metadata returned alongside a provider response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Unique request identifier.
    pub request_id: Uuid,
    /// Provider that handled the request.
    pub provider: String,
    /// Model used for generation.
    pub model: String,
    /// Response latency in milliseconds.
    pub latency_ms: Option<u64>,
    /// Arbitrary extra metadata.
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ResponseMetadata {
    /// Create a new `ResponseMetadata` with the given provider and model.
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            provider: provider.into(),
            model: model.into(),
            latency_ms: None,
            extra: HashMap::new(),
        }
    }
}

impl Default for ResponseMetadata {
    fn default() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            provider: String::new(),
            model: String::new(),
            latency_ms: None,
            extra: HashMap::new(),
        }
    }
}

/// Caches model information fetched from remote APIs.
///
/// Model IDs change frequently.  Use `fetch_models()` on the provider
/// to populate the registry, then look up models by ID.
#[derive(Debug, Clone, Default)]
pub struct ModelRegistry {
    /// Internal list of registered models.
    models: Vec<ModelInfo>,
    generation: u64,
}

/// The result of replacing a model registry snapshot.
#[derive(Debug, Clone, Default)]
pub struct ModelRegistryUpdate {
    /// Models that were not present in the previous snapshot.
    pub added: Vec<ModelInfo>,
    /// Models whose metadata changed.
    pub updated: Vec<ModelInfo>,
    /// Models that disappeared from the new snapshot.
    pub removed: Vec<ModelInfo>,
    /// Monotonically increasing snapshot number.
    pub generation: u64,
}

impl ModelRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            generation: 0,
        }
    }

    /// Merge freshly fetched models into the registry.
    pub fn update(&mut self, models: Vec<ModelInfo>) {
        for model in models {
            if let Some(existing) = self.models.iter_mut().find(|m| m.id == model.id) {
                *existing = model;
            } else {
                self.models.push(model);
            }
        }
        self.generation = self.generation.saturating_add(1);
    }

    /// Replace the registry with a fresh provider snapshot and return a diff.
    ///
    /// Unlike [`ModelRegistry::update`], this removes models that are no
    /// longer advertised. This is the operation consumers should use for
    /// periodic discovery.
    pub fn replace(&mut self, models: Vec<ModelInfo>) -> ModelRegistryUpdate {
        let mut next = Vec::new();
        for model in models {
            if !next.iter().any(|m: &ModelInfo| m.id == model.id) {
                next.push(model);
            }
        }

        let added = next
            .iter()
            .filter(|model| !self.models.iter().any(|old| old.id == model.id))
            .cloned()
            .collect();
        let updated = next
            .iter()
            .filter(|model| {
                self.models
                    .iter()
                    .find(|old| old.id == model.id)
                    .is_some_and(|old| old != *model)
            })
            .cloned()
            .collect();
        let removed = self
            .models
            .iter()
            .filter(|old| !next.iter().any(|model| model.id == old.id))
            .cloned()
            .collect();

        self.models = next;
        self.generation = self.generation.saturating_add(1);

        ModelRegistryUpdate {
            added,
            updated,
            removed,
            generation: self.generation,
        }
    }

    /// Snapshot generation. It changes after every update or replacement.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Look up a model by ID.
    pub fn get(&self, id: &str) -> Option<&ModelInfo> {
        self.models.iter().find(|m| m.id == id)
    }

    /// All known models.
    pub fn all(&self) -> &[ModelInfo] {
        &self.models
    }

    /// Filter models by a required capability.
    pub fn with_capability(&self, cap: &Capability) -> Vec<&ModelInfo> {
        self.models
            .iter()
            .filter(|m| m.capabilities.has(cap))
            .collect()
    }
}

#[cfg(test)]
mod model_registry_tests {
    use super::*;

    fn model(id: &str) -> ModelInfo {
        ModelInfo {
            id: id.into(),
            provider: "test".into(),
            display_name: id.into(),
            capabilities: CapabilitySet::new(),
            ..Default::default()
        }
    }

    #[test]
    fn replace_reports_additions_updates_removals_and_generation() {
        let mut registry = ModelRegistry::new();
        let first = registry.replace(vec![model("a"), model("b")]);
        assert_eq!(first.added.len(), 2);
        assert!(first.updated.is_empty());
        assert!(first.removed.is_empty());

        let mut changed = model("a");
        changed.description = Some("changed".into());
        let second = registry.replace(vec![changed.clone(), model("c")]);
        assert_eq!(
            second
                .added
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            ["c"]
        );
        assert_eq!(second.updated, vec![changed]);
        assert_eq!(
            second
                .removed
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            ["b"]
        );
        assert_eq!(second.generation, 2);
    }

    #[test]
    fn update_replaces_metadata_without_duplicates() {
        let mut registry = ModelRegistry::new();
        registry.update(vec![model("a")]);
        let mut changed = model("a");
        changed.description = Some("changed".into());
        registry.update(vec![changed]);
        assert_eq!(registry.all().len(), 1);
        assert_eq!(
            registry.get("a").and_then(|m| m.description.as_deref()),
            Some("changed")
        );
    }
}
