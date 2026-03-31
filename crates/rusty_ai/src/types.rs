use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capability::{Capability, CapabilitySet};

/// Reason the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCall,
    ContentFilter,
    Error,
    Unknown,
}

/// Metadata describing a model exposed by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub capabilities: CapabilitySet,
}

/// Metadata attached to an outgoing request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
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
    pub request_id: Uuid,
    pub provider: String,
    pub model: String,
    pub latency_ms: Option<u64>,
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
    models: Vec<ModelInfo>,
}

impl ModelRegistry {
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
