//! Common types for model metadata and finish reasons.
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capability::{Capability, CapabilitySet};

/// Reason the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Generation completed naturally.
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

/// Metadata describing a model exposed by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier.
    pub id: String,
    /// Provider identifier.
    pub provider: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Capabilities supported by this model.
    pub capabilities: CapabilitySet,
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
}

impl ModelRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    /// Merge freshly fetched models into the registry.
    pub fn update(&mut self, models: Vec<ModelInfo>) {
        for model in models {
            if !self.models.iter().any(|m| m.id == model.id) {
                self.models.push(model);
            }
        }
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
