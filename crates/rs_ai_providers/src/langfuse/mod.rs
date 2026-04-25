//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! LangFuse observability integration for RAI language models.
//!
//! LangFuse is an LLM observability platform that tracks:
//! - Request/response traces (latency, cost, tokens)
//! - Errors and failures
//! - User interactions
//! - Custom events and metrics
//!
//! This crate wraps any [`LanguageModel`] to automatically emit events to LangFuse.
//!
//! # Example
//!
//! ```rust,ignore
//! use rs_ai_providers::langfuse::with_langfuse;
//!
//! let model: Box<dyn LanguageModel> = /* any provider */;
//! let observable = with_langfuse(
//!     model,
//!     "langfuse_public_key",
//!     "langfuse_secret_key",
//! ).await?;
//!
//! // observable behaves exactly like the inner model, but emits LangFuse traces.
//! let result = observable.generate(prompt, options).await?;
//! ```

pub mod client;
pub mod error;

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use async_trait::async_trait;
use futures::Stream;
use pin_project_lite::pin_project;
use uuid::Uuid;

use rs_ai_core::{
    AiError, AiResult, AiStream, CapabilitySet, FinishReason, GenerateOptions, GenerateResult,
    LanguageModel, Prompt, StreamEvent, Usage,
};

pub use client::{LangfuseClient, LangfuseClientRef, LangfuseEvent};
pub use error::{LangfuseError, LangfuseResult};

// ---------------------------------------------------------------------------
// LangfuseModel
// ---------------------------------------------------------------------------

/// A [`LanguageModel`] wrapper that emits events to LangFuse.
///
/// Create one via [`LangfuseModel::new`] or the [`with_langfuse`] helper.
pub struct LangfuseModel {
    inner: Box<dyn LanguageModel>,
    client: LangfuseClientRef,
    trace_id: String,
    user_id: Option<String>,
    session_id: Option<String>,
}

impl LangfuseModel {
    /// Wrap an existing model with LangFuse observability.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying language model
    /// * `client` - The LangFuse HTTP client
    pub fn new(inner: Box<dyn LanguageModel>, client: LangfuseClientRef) -> Self {
        Self {
            inner,
            client,
            trace_id: Uuid::new_v4().to_string(),
            user_id: None,
            session_id: None,
        }
    }

    /// Set a custom trace ID for this model.
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = trace_id;
        self
    }

    /// Set the user ID for tracking.
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the session ID for tracking.
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Emit a trace event to LangFuse without blocking.
    fn emit_trace_async(&self, event: LangfuseEvent) {
        let client = self.client.clone();
        tokio::spawn(async move {
            if let Err(e) = client.emit_trace(event).await {
                tracing::warn!("Failed to emit LangFuse trace: {}", e);
            }
        });
    }

    /// Build a base LangFuse event with common fields.
    fn build_event(&self, name: &str) -> LangfuseEvent {
        let mut event = LangfuseEvent::new(name, self.inner.model_id(), self.inner.provider_id())
            .with_trace_id(self.trace_id.clone());

        if let Some(ref user_id) = self.user_id {
            event = event.with_user_id(user_id.clone());
        }
        if let Some(ref session_id) = self.session_id {
            event = event.with_session_id(session_id.clone());
        }

        event
    }
}

#[async_trait]
impl LanguageModel for LangfuseModel {
    fn model_id(&self) -> &str {
        self.inner.model_id()
    }

    fn provider_id(&self) -> &str {
        self.inner.provider_id()
    }

    fn capabilities(&self) -> &CapabilitySet {
        self.inner.capabilities()
    }

    /// Calls the inner model's `generate` and emits a trace event.
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        let start = Instant::now();
        let result = self.inner.generate(prompt, options).await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match &result {
            Ok(gen_result) => {
                let mut event = self.build_event("ai.generate").with_latency_ms(latency_ms);

                if let Some(pt) = gen_result.usage.prompt_tokens {
                    event = event.with_prompt_tokens(pt);
                }
                if let Some(ct) = gen_result.usage.completion_tokens {
                    event = event.with_completion_tokens(ct);
                }

                event = event
                    .with_finish_reason(finish_reason_str(&gen_result.finish_reason).to_string());

                self.emit_trace_async(event);
            }
            Err(_) => {
                let event = self
                    .build_event("ai.generate")
                    .with_latency_ms(latency_ms)
                    .with_finish_reason("error".to_string());

                self.emit_trace_async(event);
            }
        }

        result
    }

    /// Calls the inner model's `stream` and wraps it to emit traces.
    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let start = Instant::now();
        let inner_stream = self.inner.stream(prompt, options).await?;

        let langfuse_stream = LangfuseStream {
            inner: inner_stream,
            client: self.client.clone(),
            event_base: self.build_event("ai.stream"),
            start,
            cumulative_usage: Usage::default(),
        };

        Ok(Box::pin(langfuse_stream))
    }
}

// ---------------------------------------------------------------------------
// LangfuseStream
// ---------------------------------------------------------------------------

pin_project! {
    /// Internal stream wrapper that emits a trace when the stream ends.
    struct LangfuseStream {
        #[pin]
        inner: AiStream,
        client: LangfuseClientRef,
        event_base: LangfuseEvent,
        start: Instant,
        cumulative_usage: Usage,
    }
}

impl Stream for LangfuseStream {
    type Item = Result<StreamEvent, AiError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.inner.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(item)) => {
                match &item {
                    Ok(StreamEvent::UsageDelta { usage }) => {
                        let usage = usage.clone();
                        this.cumulative_usage.merge(&usage);
                    }
                    Ok(StreamEvent::MessageEnd {
                        finish_reason,
                        usage,
                    }) => {
                        // Merge any usage carried in the MessageEnd event.
                        if let Some(u) = usage {
                            this.cumulative_usage.merge(u);
                        }

                        let latency_ms = this.start.elapsed().as_millis() as u64;
                        let mut event = this
                            .event_base
                            .clone()
                            .with_latency_ms(latency_ms)
                            .with_finish_reason(finish_reason_str(finish_reason).to_string());

                        if let Some(pt) = this.cumulative_usage.prompt_tokens {
                            event = event.with_prompt_tokens(pt);
                        }
                        if let Some(ct) = this.cumulative_usage.completion_tokens {
                            event = event.with_completion_tokens(ct);
                        }

                        let client = this.client.clone();
                        tokio::spawn(async move {
                            if let Err(e) = client.emit_trace(event).await {
                                tracing::warn!("Failed to emit LangFuse stream trace: {}", e);
                            }
                        });
                    }
                    _ => {}
                }
                Poll::Ready(Some(item))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Convert a [`FinishReason`] to a string representation.
fn finish_reason_str(reason: &FinishReason) -> &'static str {
    match reason {
        FinishReason::Stop => "stop",
        FinishReason::Length => "length",
        FinishReason::ToolCall => "tool_call",
        FinishReason::ContentFilter => "content_filter",
        FinishReason::Error => "error",
        FinishReason::Unknown => "unknown",
    }
}

/// Wrap any [`LanguageModel`] with LangFuse observability.
///
/// # Arguments
///
/// * `model` - The underlying language model
/// * `public_key` - LangFuse public API key
/// * `secret_key` - LangFuse secret API key
///
/// # Errors
///
/// Returns an error if the API keys are invalid or the client cannot be created.
///
/// # Example
///
/// ```rust,ignore
/// let model: Box<dyn LanguageModel> = rs_ai::claude("claude-3-5-sonnet").await?;
/// let observable = with_langfuse(model, "pub_key", "secret_key").await?;
/// let result = observable.generate(prompt, options).await?;
/// ```
pub async fn with_langfuse(
    model: Box<dyn LanguageModel>,
    public_key: impl Into<String>,
    secret_key: impl Into<String>,
) -> LangfuseResult<LangfuseModel> {
    let client = LangfuseClient::new(public_key, secret_key);
    Ok(LangfuseModel::new(model, std::sync::Arc::new(client)))
}

/// Wrap a model with LangFuse observability using environment variables.
///
/// Looks for `LANGFUSE_PUBLIC_KEY` and `LANGFUSE_SECRET_KEY` environment variables.
pub async fn with_langfuse_from_env(
    model: Box<dyn LanguageModel>,
) -> LangfuseResult<LangfuseModel> {
    let client = LangfuseClient::from_env()?;
    Ok(LangfuseModel::new(model, std::sync::Arc::new(client)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use rs_ai_core::{Capability, CapabilitySet, FinishReason, ResponseMetadata};
    use std::sync::Arc;

    struct MockModel {
        model_id: String,
        provider_id: String,
        capabilities: CapabilitySet,
    }

    impl MockModel {
        fn new() -> Self {
            let capabilities = CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::Streaming);
            Self {
                model_id: "mock-model".to_string(),
                provider_id: "mock-provider".to_string(),
                capabilities,
            }
        }
    }

    #[async_trait]
    impl LanguageModel for MockModel {
        fn model_id(&self) -> &str {
            &self.model_id
        }

        fn provider_id(&self) -> &str {
            &self.provider_id
        }

        fn capabilities(&self) -> &CapabilitySet {
            &self.capabilities
        }

        async fn generate(
            &self,
            _prompt: Prompt,
            _options: GenerateOptions,
        ) -> AiResult<GenerateResult> {
            Ok(GenerateResult {
                text: Some("Hello, LangFuse!".to_string()),
                tool_calls: vec![],
                finish_reason: FinishReason::Stop,
                usage: Usage {
                    prompt_tokens: Some(10),
                    completion_tokens: Some(5),
                    total_tokens: Some(15),
                },
                metadata: ResponseMetadata::default(),
            })
        }

        async fn stream(&self, _prompt: Prompt, _options: GenerateOptions) -> AiResult<AiStream> {
            use futures::stream;
            let events: Vec<Result<StreamEvent, AiError>> = vec![
                Ok(StreamEvent::TextDelta {
                    delta: "Hello".to_string(),
                }),
                Ok(StreamEvent::UsageDelta {
                    usage: Usage {
                        prompt_tokens: Some(10),
                        completion_tokens: None,
                        total_tokens: None,
                    },
                }),
                Ok(StreamEvent::MessageEnd {
                    finish_reason: FinishReason::Stop,
                    usage: Some(Usage {
                        prompt_tokens: None,
                        completion_tokens: Some(5),
                        total_tokens: Some(15),
                    }),
                }),
            ];
            Ok(Box::pin(stream::iter(events)))
        }
    }

    #[tokio::test]
    async fn test_langfuse_model_wraps_generate() {
        let mock = Box::new(MockModel::new());
        let client = Arc::new(LangfuseClient::new("test_pub", "test_secret"));
        let wrapped = LangfuseModel::new(mock, client);

        let result = wrapped
            .generate(Prompt::from("test"), GenerateOptions::default())
            .await
            .expect("generate should succeed");

        assert_eq!(result.text.as_deref(), Some("Hello, LangFuse!"));
        assert_eq!(result.usage.prompt_tokens, Some(10));
        assert_eq!(result.usage.completion_tokens, Some(5));
    }

    #[tokio::test]
    async fn test_langfuse_model_wraps_stream() {
        let mock = Box::new(MockModel::new());
        let client = Arc::new(LangfuseClient::new("test_pub", "test_secret"));
        let wrapped = LangfuseModel::new(mock, client);

        let stream = wrapped
            .stream(Prompt::from("test"), GenerateOptions::default())
            .await
            .expect("stream should succeed");

        let events: Vec<_> = stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("no stream errors");

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], StreamEvent::TextDelta { .. }));
        assert!(matches!(events[1], StreamEvent::UsageDelta { .. }));
        assert!(matches!(events[2], StreamEvent::MessageEnd { .. }));
    }

    #[test]
    fn test_langfuse_model_delegates_model_id() {
        let mock = Box::new(MockModel::new());
        let client = Arc::new(LangfuseClient::new("test_pub", "test_secret"));
        let wrapped = LangfuseModel::new(mock, client);

        assert_eq!(wrapped.model_id(), "mock-model");
        assert_eq!(wrapped.provider_id(), "mock-provider");
    }

    #[test]
    fn test_langfuse_model_with_trace_id() {
        let mock = Box::new(MockModel::new());
        let client = Arc::new(LangfuseClient::new("test_pub", "test_secret"));
        let wrapped = LangfuseModel::new(mock, client).with_trace_id("custom-trace-id".to_string());

        assert_eq!(wrapped.trace_id, "custom-trace-id");
    }

    #[test]
    fn test_langfuse_model_with_user_id() {
        let mock = Box::new(MockModel::new());
        let client = Arc::new(LangfuseClient::new("test_pub", "test_secret"));
        let wrapped = LangfuseModel::new(mock, client).with_user_id("user123".to_string());

        assert_eq!(wrapped.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_langfuse_model_with_session_id() {
        let mock = Box::new(MockModel::new());
        let client = Arc::new(LangfuseClient::new("test_pub", "test_secret"));
        let wrapped = LangfuseModel::new(mock, client).with_session_id("session456".to_string());

        assert_eq!(wrapped.session_id, Some("session456".to_string()));
    }

    #[test]
    fn test_finish_reason_str() {
        assert_eq!(finish_reason_str(&FinishReason::Stop), "stop");
        assert_eq!(finish_reason_str(&FinishReason::Length), "length");
        assert_eq!(finish_reason_str(&FinishReason::ToolCall), "tool_call");
        assert_eq!(
            finish_reason_str(&FinishReason::ContentFilter),
            "content_filter"
        );
        assert_eq!(finish_reason_str(&FinishReason::Error), "error");
        assert_eq!(finish_reason_str(&FinishReason::Unknown), "unknown");
    }
}
