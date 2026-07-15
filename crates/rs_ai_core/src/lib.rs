//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Core traits, types, and abstractions for the Rust AI SDK (RAI).

pub mod capability;
pub mod content;
pub mod embedding;
pub mod error;
pub mod message;
pub mod middleware;
pub mod model;
pub mod prompt;
pub mod provider;
pub mod router;
pub mod schema;
pub mod stream;
pub mod structured;
pub mod tool;
pub mod types;
pub mod usage;

pub mod cache;
pub mod observability;
// middleware and ui_stream are already declared above from the old traits crate,
// but we need to make sure they point to the right thing. The old traits crate
// had `pub mod middleware` which was a placeholder; now it contains the impls.
// ui_stream is a new top-level module.
pub mod ui_stream;

// Re-exports for convenience.
pub use capability::{Capability, CapabilitySet};
pub use content::{ContentPart, FileData, ImageData, ImageDetail};
pub use embedding::cosine_similarity;
pub use error::{AiError, AiResult};
pub use message::{Message, Role};
pub use model::{
    EmbeddingModel, GenerateOptions, ImageGenerationOptions, ImageModel, LanguageModel, Middleware,
    MiddlewareNext, ProviderInfo, ReasoningEffort, RealtimeEvent, RealtimeSession,
    SpeechToTextModel, TextToSpeechModel, ThinkingConfig, VideoGenerationOptions, VideoModel,
};
pub use prompt::Prompt;
pub use provider::Provider;
pub use router::{Route, Router};
pub use schema::OutputSchema;
pub use stream::{AiStream, StreamCollector, StreamEvent, SyntheticStreamer};
pub use structured::{
    AudioResult, EmbeddingResult, GenerateResult, GeneratedFile, ImageResult, ObjectResult,
    TranscriptionResult, TtsOptions, VideoResult,
};
pub use tool::{ToolCallRequest, ToolCallResult, ToolChoice, ToolDefinition, ToolSet};
pub use types::{FinishReason, ModelInfo, ModelRegistry, RequestMetadata, ResponseMetadata};
pub use usage::Usage;

// Re-exports from merged crates.
pub use cache::{CacheConfig, CacheTTL};
pub use observability::{with_observability, ObservableModel};

/// Generate text from a language model with default options.
///
/// Returns an error if the model responds with no text content (e.g. the
/// model responded with only tool calls).
pub async fn generate_text(
    model: &dyn LanguageModel,
    prompt: impl Into<Prompt>,
) -> AiResult<String> {
    let result = model
        .generate(prompt.into(), GenerateOptions::default())
        .await?;
    result.text.ok_or_else(|| AiError::ProviderError {
        provider: model.provider_id().to_string(),
        status: None,
        message: "Response contained no text content (model responded with tool calls only)"
            .to_string(),
    })
}

/// Stream text from a language model with default options.
pub async fn stream_text(
    model: &dyn LanguageModel,
    prompt: impl Into<Prompt>,
) -> AiResult<AiStream> {
    model
        .stream(prompt.into(), GenerateOptions::default())
        .await
}

/// Embed texts using an embedding model.
pub async fn embed(model: &dyn EmbeddingModel, texts: Vec<String>) -> AiResult<EmbeddingResult> {
    model.embed(texts).await
}

/// Generate images from a text prompt using an image model.
/// Equivalent to Vercel AI SDK `generateImage()`.
pub async fn generate_image(
    model: &dyn ImageModel,
    prompt: &str,
    options: ImageGenerationOptions,
) -> AiResult<ImageResult> {
    model.generate_image(prompt, options).await
}

/// Generate a video from a text prompt using a video model.
/// Equivalent to Vercel AI SDK `experimental_generateVideo()`.
pub async fn generate_video(
    model: &dyn VideoModel,
    prompt: &str,
    options: VideoGenerationOptions,
) -> AiResult<VideoResult> {
    model.generate_video(prompt, options).await
}
