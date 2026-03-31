//! Core traits, types, and abstractions for the Rusty AI SDK.

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

// Re-exports for convenience.
pub use capability::{Capability, CapabilitySet};
pub use content::{ContentPart, FileData, ImageData, ImageDetail};
pub use embedding::cosine_similarity;
pub use error::{AiError, AiResult};
pub use message::{Message, Role};
pub use model::{
    EmbeddingModel, GenerateOptions, LanguageModel, Middleware, MiddlewareNext, ProviderInfo,
    ReasoningEffort, SpeechToTextModel, TextToSpeechModel, ThinkingConfig,
};
pub use prompt::Prompt;
pub use provider::Provider;
pub use router::{Route, Router};
pub use schema::OutputSchema;
pub use stream::{AiStream, StreamCollector, StreamEvent, SyntheticStreamer};
pub use structured::{
    AudioResult, EmbeddingResult, GenerateResult, ObjectResult, TranscriptionResult, TtsOptions,
};
pub use tool::{ToolCallRequest, ToolCallResult, ToolChoice, ToolDefinition, ToolSet};
pub use types::{FinishReason, ModelInfo, ModelRegistry, RequestMetadata, ResponseMetadata};
pub use usage::Usage;

/// Generate text from a language model with default options.
pub async fn generate_text(
    model: &dyn LanguageModel,
    prompt: impl Into<Prompt>,
) -> AiResult<String> {
    let result = model
        .generate(prompt.into(), GenerateOptions::default())
        .await?;
    result
        .text
        .ok_or(AiError::Serialization("No text in response".into()))
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
