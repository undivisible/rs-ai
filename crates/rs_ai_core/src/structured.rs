//! Structured generation result types.
use serde::{Deserialize, Serialize};

use crate::tool::{ToolCallRequest, ToolCallResult};
use crate::types::{FinishReason, ResponseMetadata};
use crate::usage::Usage;

/// A single step in a multi-turn agent loop (Vercel's `StepResult`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step number (0-indexed).
    pub step_number: u32,
    /// Text generated in this step.
    pub text: Option<String>,
    /// Tool calls made in this step.
    #[serde(default)]
    pub tool_calls: Vec<ToolCallRequest>,
    /// Tool results returned in this step.
    #[serde(default)]
    pub tool_results: Vec<ToolCallResult>,
    /// Why this step finished.
    pub finish_reason: FinishReason,
    /// Token usage for this step.
    pub usage: Usage,
    /// Reasoning content extracted from the model response, if any.
    #[serde(default)]
    pub reasoning: Option<String>,
}

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
    /// Per-step breakdown when using agent loop (maxSteps > 1).
    #[serde(default)]
    pub steps: Vec<StepResult>,
    /// Reasoning content extracted from the model response, if any.
    #[serde(default)]
    pub reasoning: Option<String>,
}

impl Default for GenerateResult {
    fn default() -> Self {
        Self {
            text: None,
            tool_calls: Vec::new(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata::default(),
            steps: Vec::new(),
            reasoning: None,
        }
    }
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
    /// Transcribed text.
    pub text: String,
    /// Detected language, if available.
    pub language: Option<String>,
    /// Duration of the audio in seconds.
    pub duration_seconds: Option<f64>,
    /// Token usage information.
    pub usage: Usage,
}

/// Result of text-to-speech synthesis.
#[derive(Debug, Clone)]
pub struct AudioResult {
    /// Synthesized audio bytes.
    pub audio: Vec<u8>,
    /// MIME type of the audio.
    pub mime_type: String,
    /// Token usage information.
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

/// A generated file (image, video, audio) matching Vercel's `GeneratedFile`.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Base64-encoded file data.
    pub base64: String,
    /// Raw file bytes.
    pub bytes: Vec<u8>,
    /// IANA media type (e.g. "image/png", "video/mp4").
    pub media_type: String,
}

/// Result of an image generation call, matching Vercel's `GenerateImageResult`.
#[derive(Debug, Clone)]
pub struct ImageResult {
    /// The first generated image.
    pub image: GeneratedFile,
    /// All generated images.
    pub images: Vec<GeneratedFile>,
    /// Token usage, if available.
    pub usage: Usage,
    /// Response metadata.
    pub metadata: ResponseMetadata,
}

/// Result of a video generation call, matching Vercel's `GenerateVideoResult`.
#[derive(Debug, Clone)]
pub struct VideoResult {
    /// The first generated video.
    pub video: GeneratedFile,
    /// All generated videos.
    pub videos: Vec<GeneratedFile>,
    /// Token usage, if available.
    pub usage: Usage,
    /// Response metadata.
    pub metadata: ResponseMetadata,
}

/// A single reranked document result, matching Vercel's `RerankedDocument`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RerankedDocument {
    /// Original index in the input documents array.
    pub index: usize,
    /// Relevance score (0.0 to 1.0).
    pub score: f64,
    /// The document text (if `return_documents` was true).
    pub document: Option<String>,
}

/// Result of a rerank call, matching Vercel's `RerankResult`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RerankResult {
    /// Reranked documents in order of relevance (most relevant first).
    pub results: Vec<RerankedDocument>,
    /// Token usage.
    pub usage: Usage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_result_creation() {
        let step = StepResult {
            step_number: 0,
            text: Some("hello".into()),
            tool_calls: vec![],
            tool_results: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            reasoning: None,
        };
        assert_eq!(step.step_number, 0);
        assert_eq!(step.text.as_deref(), Some("hello"));
    }

    #[test]
    fn test_generate_result_default() {
        let result = GenerateResult::default();
        assert_eq!(result.finish_reason, FinishReason::Stop);
        assert!(result.text.is_none());
    }

    #[test]
    fn test_generate_result_with_steps() {
        let step = StepResult {
            step_number: 0,
            text: Some("step1".into()),
            tool_calls: vec![],
            tool_results: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            reasoning: None,
        };
        let result = GenerateResult {
            text: Some("final".into()),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata::default(),
            steps: vec![step],
            reasoning: None,
        };
        assert_eq!(result.text.as_deref(), Some("final"));
        assert_eq!(result.steps.len(), 1);
    }

    #[test]
    fn test_image_result_creation() {
        let file = GeneratedFile {
            base64: "AAAA".into(),
            bytes: vec![0, 0, 0, 0],
            media_type: "image/png".into(),
        };
        let result = ImageResult {
            image: file.clone(),
            images: vec![file],
            usage: Usage::default(),
            metadata: ResponseMetadata::default(),
        };
        assert_eq!(result.image.media_type, "image/png");
        assert_eq!(result.images.len(), 1);
    }

    #[test]
    fn test_video_result_creation() {
        let file = GeneratedFile {
            base64: "AAAA".into(),
            bytes: vec![0, 0, 0, 0],
            media_type: "video/mp4".into(),
        };
        let result = VideoResult {
            video: file.clone(),
            videos: vec![file],
            usage: Usage::default(),
            metadata: ResponseMetadata::default(),
        };
        assert_eq!(result.video.media_type, "video/mp4");
        assert_eq!(result.videos.len(), 1);
    }

    #[test]
    fn test_reranked_document_serde() {
        let doc = RerankedDocument {
            index: 0,
            score: 0.95,
            document: Some("text".into()),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let back: RerankedDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(back.index, 0);
        assert!((back.score - 0.95).abs() < 1e-6);
        assert_eq!(back.document, Some("text".into()));
    }

    #[test]
    fn test_rerank_result() {
        let result = RerankResult {
            results: vec![RerankedDocument {
                index: 0,
                score: 0.8,
                document: None,
            }],
            usage: Usage::default(),
        };
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].index, 0);
    }

    #[test]
    fn test_object_result() {
        let obj: ObjectResult<i32> = ObjectResult {
            object: 42,
            text: "42".into(),
            usage: Usage::default(),
            metadata: ResponseMetadata::default(),
        };
        assert_eq!(obj.object, 42);
        assert_eq!(obj.text, "42");
    }
}
