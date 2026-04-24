use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::stream;

use rs_ai_traits::capability::{Capability, CapabilitySet};
use rs_ai_traits::error::{AiError, AiResult};
use rs_ai_traits::model::{EmbeddingModel, GenerateOptions, LanguageModel};
use rs_ai_traits::prompt::Prompt;
use rs_ai_traits::stream::{AiStream, StreamEvent, SyntheticStreamer};
use rs_ai_traits::structured::{EmbeddingResult, GenerateResult};
use rs_ai_traits::tool::ToolCallRequest;
use rs_ai_traits::types::{FinishReason, ResponseMetadata};
use rs_ai_traits::usage::Usage;

/// A pre-configured response for the mock model to return.
pub enum MockResponse {
    /// Return a plain text response.
    Text(String),
    /// Return a set of tool calls.
    ToolCalls(Vec<ToolCallRequest>),
    /// Return an error.
    Error(AiError),
    /// Return a JSON object (serialised into the text field).
    Object(serde_json::Value),
}

/// A recorded invocation of the mock model.
#[derive(Debug, Clone)]
pub struct RecordedCall {
    /// The prompt that was passed to the model.
    pub prompt: Prompt,
    /// The options that were passed to the model.
    pub options: GenerateOptions,
    /// When the call was made.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A mock language model for testing.
///
/// Responses are consumed in FIFO order. When no responses remain the model
/// returns an error.
pub struct MockLanguageModel {
    id: String,
    provider: String,
    capabilities: CapabilitySet,
    responses: Arc<Mutex<Vec<MockResponse>>>,
    calls: Arc<Mutex<Vec<RecordedCall>>>,
}

impl MockLanguageModel {
    /// Create a new mock model with the given identifier.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            provider: "mock".to_owned(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::Streaming)
                .with(Capability::ToolCalling),
            responses: Arc::new(Mutex::new(Vec::new())),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Queue a response (builder style).
    pub fn with_response(self, response: MockResponse) -> Self {
        self.responses.lock().unwrap().push(response);
        self
    }

    /// Queue a text response (builder style).
    pub fn with_text(self, text: &str) -> Self {
        self.with_response(MockResponse::Text(text.to_owned()))
    }

    /// Queue an error response (builder style).
    pub fn with_error(self, error: AiError) -> Self {
        self.with_response(MockResponse::Error(error))
    }

    /// Queue a tool-calls response (builder style).
    pub fn with_tool_calls(self, calls: Vec<ToolCallRequest>) -> Self {
        self.with_response(MockResponse::ToolCalls(calls))
    }

    /// Queue a JSON object response (builder style).
    pub fn with_object(self, value: serde_json::Value) -> Self {
        self.with_response(MockResponse::Object(value))
    }

    /// Override the provider name (builder style).
    pub fn with_provider(mut self, provider: &str) -> Self {
        self.provider = provider.to_owned();
        self
    }

    /// Override the capability set (builder style).
    pub fn with_capabilities(mut self, capabilities: CapabilitySet) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Return a snapshot of all recorded calls.
    pub fn calls(&self) -> Vec<RecordedCall> {
        self.calls.lock().unwrap().clone()
    }

    /// Return the number of times this model was invoked.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Get the model identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the provider name.
    pub fn provider(&self) -> &str {
        &self.provider
    }

    // ------- internal helpers -------

    fn record_call(&self, prompt: &Prompt, options: &GenerateOptions) {
        self.calls.lock().unwrap().push(RecordedCall {
            prompt: prompt.clone(),
            options: options.clone(),
            timestamp: chrono::Utc::now(),
        });
    }

    fn next_response(&self) -> MockResponse {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            MockResponse::Error(AiError::StreamError {
                message: "MockLanguageModel: no more queued responses".into(),
            })
        } else {
            responses.remove(0)
        }
    }

    fn response_to_result(&self, response: MockResponse) -> AiResult<GenerateResult> {
        match response {
            MockResponse::Text(text) => Ok(GenerateResult {
                text: Some(text),
                tool_calls: Vec::new(),
                finish_reason: FinishReason::Stop,
                usage: Usage::default(),
                metadata: ResponseMetadata::default(),
            }),
            MockResponse::ToolCalls(calls) => Ok(GenerateResult {
                text: None,
                tool_calls: calls,
                finish_reason: FinishReason::ToolCall,
                usage: Usage::default(),
                metadata: ResponseMetadata::default(),
            }),
            MockResponse::Object(value) => {
                let text = serde_json::to_string(&value)
                    .map_err(|e| AiError::Serialization(e.to_string()))?;
                Ok(GenerateResult {
                    text: Some(text),
                    tool_calls: Vec::new(),
                    finish_reason: FinishReason::Stop,
                    usage: Usage::default(),
                    metadata: ResponseMetadata::default(),
                })
            }
            MockResponse::Error(err) => Err(err),
        }
    }

    fn response_to_stream(&self, response: MockResponse) -> AiResult<AiStream> {
        match response {
            MockResponse::Text(text) => Ok(SyntheticStreamer::stream(text, 20)),
            MockResponse::ToolCalls(calls) => {
                let mut events: Vec<Result<StreamEvent, AiError>> = Vec::new();
                events.push(Ok(StreamEvent::MessageStart {
                    message_id: uuid::Uuid::new_v4().to_string(),
                }));
                for call in calls {
                    events.push(Ok(StreamEvent::ToolCallStart {
                        call_id: call.id.clone(),
                        tool_name: call.name.clone(),
                    }));
                    events.push(Ok(StreamEvent::ToolCallEnd {
                        call_id: call.id.clone(),
                        arguments: call.arguments.clone(),
                    }));
                }
                events.push(Ok(StreamEvent::MessageEnd {
                    finish_reason: FinishReason::ToolCall,
                    usage: None,
                }));
                Ok(Box::pin(stream::iter(events)))
            }
            MockResponse::Object(value) => {
                let text = serde_json::to_string(&value)
                    .map_err(|e| AiError::Serialization(e.to_string()))?;
                Ok(SyntheticStreamer::stream(text, 20))
            }
            MockResponse::Error(err) => Err(err),
        }
    }
}

#[async_trait]
impl LanguageModel for MockLanguageModel {
    fn model_id(&self) -> &str {
        &self.id
    }

    fn provider_id(&self) -> &str {
        &self.provider
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        self.record_call(&prompt, &options);
        let response = self.next_response();
        self.response_to_result(response)
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        self.record_call(&prompt, &options);
        let response = self.next_response();
        self.response_to_stream(response)
    }
}

// ---------------------------------------------------------------------------
// Mock embedding model
// ---------------------------------------------------------------------------

/// A mock embedding model for testing.
///
/// Returns pre-configured embedding vectors in FIFO order. When the queue is
/// exhausted it produces zero vectors.
pub struct MockEmbeddingModel {
    id: String,
    provider: String,
    dims: usize,
    embeddings: Arc<Mutex<Vec<Vec<f64>>>>,
}

impl MockEmbeddingModel {
    /// Create a new mock embedding model.
    pub fn new(id: &str, dims: usize) -> Self {
        Self {
            id: id.to_owned(),
            provider: "mock".to_owned(),
            dims,
            embeddings: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Queue embedding vectors (builder style).
    pub fn with_embeddings(self, embeddings: Vec<Vec<f64>>) -> Self {
        let mut store = self.embeddings.lock().unwrap();
        store.extend(embeddings);
        drop(store);
        self
    }

    /// Get the model identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the provider name.
    pub fn provider(&self) -> &str {
        &self.provider
    }
}

#[async_trait]
impl EmbeddingModel for MockEmbeddingModel {
    fn model_id(&self) -> &str {
        &self.id
    }

    fn provider_id(&self) -> &str {
        &self.provider
    }

    fn dimensions(&self) -> Option<usize> {
        Some(self.dims)
    }

    async fn embed(&self, texts: Vec<String>) -> AiResult<EmbeddingResult> {
        let mut store = self.embeddings.lock().unwrap();
        let mut result = Vec::with_capacity(texts.len());
        for _ in &texts {
            if store.is_empty() {
                // Return zero vectors when queue is exhausted.
                result.push(vec![0.0f64; self.dims]);
            } else {
                result.push(store.remove(0));
            }
        }
        Ok(EmbeddingResult {
            embeddings: result,
            usage: Usage::default(),
        })
    }
}
