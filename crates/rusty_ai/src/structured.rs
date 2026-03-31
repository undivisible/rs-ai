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

/// Result of a speech-to-text transcription.
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_seconds: Option<f64>,
    pub usage: Usage,
}

/// Result of text-to-speech synthesis.
#[derive(Debug, Clone)]
pub struct AudioResult {
    pub audio: Vec<u8>,
    pub mime_type: String,
    pub usage: Usage,
}

/// Options for text-to-speech.
#[derive(Debug, Clone, Default)]
pub struct TtsOptions {
    /// Speech speed multiplier (e.g. 1.0 = normal).
    pub speed: Option<f64>,
    /// Output audio format (e.g. "mp3", "opus", "aac", "flac", "wav", "pcm").
    pub response_format: Option<String>,
}
