//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Observability wrapper for RAI language models.
//!
//! Wraps any [`LanguageModel`] to automatically record tracing spans with:
//! - `ai.provider` — provider identifier
//! - `ai.model` — model identifier
//! - `ai.usage.prompt_tokens` — prompt token count
//! - `ai.usage.completion_tokens` — completion token count
//! - `ai.finish_reason` — finish reason
//! - `ai.latency_ms` — wall-clock latency in milliseconds
//!
//! # Example
//!
//! ```rust,ignore
//! use rs_ai_core::observability::with_observability;
//!
//! let model: Box<dyn LanguageModel> = /* any provider */;
//! let observable = with_observability(model);
//! // observable behaves exactly like the inner model, but emits tracing spans.
//! ```

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use async_trait::async_trait;
use futures::Stream;
use pin_project_lite::pin_project;
use tracing::Span;

use crate::{
    AiError, AiResult, AiStream, CapabilitySet, FinishReason, GenerateOptions, GenerateResult,
    LanguageModel, Prompt, StreamEvent, Usage,
};

// ---------------------------------------------------------------------------
// ObservableModel
// ---------------------------------------------------------------------------

/// A [`LanguageModel`] wrapper that emits tracing spans for each call.
///
/// Create one via [`ObservableModel::new`] or the [`with_observability`] helper.
pub struct ObservableModel {
    inner: Box<dyn LanguageModel>,
    record_content: bool,
}

impl ObservableModel {
    /// Wrap an existing model with observability instrumentation.
    pub fn new(inner: Box<dyn LanguageModel>) -> Self {
        Self {
            inner,
            record_content: false,
        }
    }

    /// Set whether to record request/response content in trace spans.
    pub fn with_record_content(mut self, record: bool) -> Self {
        self.record_content = record;
        self
    }
}

#[async_trait]
impl LanguageModel for ObservableModel {
    fn model_id(&self) -> &str {
        self.inner.model_id()
    }

    fn provider_id(&self) -> &str {
        self.inner.provider_id()
    }

    fn capabilities(&self) -> &CapabilitySet {
        self.inner.capabilities()
    }

    /// Calls the inner model's `generate`, records a tracing span with usage
    /// and latency information, and returns the result unchanged.
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        let span = tracing::info_span!(
            "ai.generate",
            ai.provider = self.inner.provider_id(),
            ai.model = self.inner.model_id(),
            ai.usage.prompt_tokens = tracing::field::Empty,
            ai.usage.completion_tokens = tracing::field::Empty,
            ai.finish_reason = tracing::field::Empty,
            ai.latency_ms = tracing::field::Empty,
        );
        let _enter = span.enter();

        let start = Instant::now();
        let result = self.inner.generate(prompt, options).await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match &result {
            Ok(gen) => {
                record_usage_and_latency(&span, &gen.usage, &gen.finish_reason, latency_ms);
            }
            Err(_) => {
                span.record("ai.latency_ms", latency_ms);
            }
        }

        result
    }

    /// Calls the inner model's `stream` and wraps the returned stream so that
    /// a tracing span is completed with cumulative usage and latency when a
    /// [`StreamEvent::MessageEnd`] event arrives.
    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let span = tracing::info_span!(
            "ai.stream",
            ai.provider = self.inner.provider_id(),
            ai.model = self.inner.model_id(),
            ai.usage.prompt_tokens = tracing::field::Empty,
            ai.usage.completion_tokens = tracing::field::Empty,
            ai.finish_reason = tracing::field::Empty,
            ai.latency_ms = tracing::field::Empty,
        );

        let start = Instant::now();
        let inner_stream = self.inner.stream(prompt, options).await?;

        let observable_stream = ObservableStream {
            inner: inner_stream,
            span,
            start,
            cumulative_usage: Usage::default(),
        };

        Ok(Box::pin(observable_stream))
    }
}

// ---------------------------------------------------------------------------
// Helper: record span fields
// ---------------------------------------------------------------------------

fn record_usage_and_latency(
    span: &Span,
    usage: &Usage,
    finish_reason: &FinishReason,
    latency_ms: u64,
) {
    if let Some(pt) = usage.prompt_tokens {
        span.record("ai.usage.prompt_tokens", pt);
    }
    if let Some(ct) = usage.completion_tokens {
        span.record("ai.usage.completion_tokens", ct);
    }
    span.record("ai.finish_reason", finish_reason_str(finish_reason));
    span.record("ai.latency_ms", latency_ms);
}

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

// ---------------------------------------------------------------------------
// ObservableStream
// ---------------------------------------------------------------------------

pin_project! {
    /// Internal stream wrapper that records a tracing span when the stream ends.
    struct ObservableStream {
        #[pin]
        inner: AiStream,
        span: Span,
        start: Instant,
        cumulative_usage: Usage,
    }
}

impl Stream for ObservableStream {
    type Item = Result<StreamEvent, AiError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // Poll the inner stream inside the span so child events are attributed to it.
        let poll_result = {
            let _enter = this.span.enter();
            this.inner.poll_next(cx)
        };

        match poll_result {
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
                        let finish_reason = finish_reason.clone();
                        let cumulative = this.cumulative_usage.clone();
                        record_usage_and_latency(
                            this.span,
                            &cumulative,
                            &finish_reason,
                            latency_ms,
                        );
                    }
                    _ => {}
                }
                Poll::Ready(Some(item))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public helper
// ---------------------------------------------------------------------------

/// Wrap any [`LanguageModel`] with observability instrumentation.
///
/// This is a convenience function equivalent to [`ObservableModel::new`].
pub fn with_observability(model: Box<dyn LanguageModel>) -> ObservableModel {
    ObservableModel::new(model)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Capability, CapabilitySet, FinishReason, GenerateResult, ResponseMetadata, Usage};
    use futures::StreamExt;

    // Minimal in-process mock that returns a fixed response without any
    // network calls.
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
                text: Some("Hello, world!".to_string()),
                tool_calls: vec![],
                finish_reason: FinishReason::Stop,
                usage: Usage {
                    prompt_tokens: Some(10),
                    completion_tokens: Some(5),
                    total_tokens: Some(15),
                },
                metadata: ResponseMetadata::default(),
                steps: Vec::new(),
                reasoning: None,
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
    async fn test_generate_delegates_to_inner() {
        let mock = Box::new(MockModel::new());
        let observable = ObservableModel::new(mock);

        let result = observable
            .generate(Prompt::from("test"), GenerateOptions::default())
            .await
            .expect("generate should succeed");

        assert_eq!(result.text.as_deref(), Some("Hello, world!"));
        assert_eq!(result.usage.prompt_tokens, Some(10));
        assert_eq!(result.usage.completion_tokens, Some(5));
        assert_eq!(result.finish_reason, FinishReason::Stop);
    }

    #[tokio::test]
    async fn test_stream_collects_all_events() {
        let mock = Box::new(MockModel::new());
        let observable = ObservableModel::new(mock);

        let stream = observable
            .stream(Prompt::from("test"), GenerateOptions::default())
            .await
            .expect("stream should succeed");

        let events: Vec<_> = stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("no stream errors");

        // Expect TextDelta, UsageDelta, MessageEnd
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], StreamEvent::TextDelta { .. }));
        assert!(matches!(events[1], StreamEvent::UsageDelta { .. }));
        assert!(matches!(events[2], StreamEvent::MessageEnd { .. }));
    }

    #[tokio::test]
    async fn test_model_id_and_provider_id() {
        let mock = Box::new(MockModel::new());
        let observable = ObservableModel::new(mock);

        assert_eq!(observable.model_id(), "mock-model");
        assert_eq!(observable.provider_id(), "mock-provider");
    }

    #[tokio::test]
    async fn test_with_observability_helper() {
        let mock = Box::new(MockModel::new()) as Box<dyn LanguageModel>;
        let observable = with_observability(mock);

        let result = observable
            .generate(Prompt::from("test"), GenerateOptions::default())
            .await
            .expect("generate should succeed");

        assert_eq!(result.text.as_deref(), Some("Hello, world!"));
    }
}
