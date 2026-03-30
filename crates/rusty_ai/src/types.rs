use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capability::CapabilitySet;

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
