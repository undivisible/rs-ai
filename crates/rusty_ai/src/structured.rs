use serde::{Deserialize, Serialize};

use crate::tool::ToolCallRequest;
use crate::types::{FinishReason, ResponseMetadata};
use crate::usage::Usage;

/// The result of a non-streaming generate call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    /// Generated text (may be `None` when the model only produced tool calls).
    pub text: Option<String>,
    /// Tool calls requested by the model.
    #[serde(default)]
    pub tool_calls: Vec<ToolCallRequest>,
    /// Reason the model stopped generating.
    pub finish_reason: FinishReason,
    /// Token usage information.
    pub usage: Usage,
    /// Provider-specific response metadata.
    #[serde(default)]
    pub metadata: ResponseMetadata,
}

/// The result of a structured object generation.
#[derive(Debug, Clone)]
pub struct ObjectResult<T> {
    /// The parsed object.
    pub object: T,
    /// The raw text that was parsed.
    pub text: String,
    /// Token usage information.
    pub usage: Usage,
    /// Provider-specific response metadata.
    pub metadata: ResponseMetadata,
}

/// The result of an embedding call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    /// The embedding vectors.
    pub embeddings: Vec<Vec<f64>>,
    /// Token usage information.
    pub usage: Usage,
}
