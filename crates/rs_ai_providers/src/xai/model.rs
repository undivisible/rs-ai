use async_trait::async_trait;
use futures::stream::StreamExt;
use rs_ai_core::CacheConfig;
use rs_ai_core::{
    AiError, AiResult, Capability, CapabilitySet, FinishReason, GenerateOptions, GenerateResult,
    LanguageModel, Prompt, StreamEvent, Usage,
};

use super::client::{ChatCompletionRequest, Message, XaiClient};

/// xAI Grok model.
#[derive(Clone)]
pub struct XaiModel {
    model_id: String,
    client: std::sync::Arc<XaiClient>,
    capabilities: CapabilitySet,
    cache_config: Option<CacheConfig>,
}

impl XaiModel {
    /// Create a new xAI model instance.
    pub fn new(model_id: String, client: std::sync::Arc<XaiClient>) -> Self {
        let capabilities = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::Streaming);

        Self {
            model_id,
            client,
            capabilities,
            cache_config: None,
        }
    }

    /// Set cache configuration for this model.
    pub fn set_cache(&mut self, config: CacheConfig) -> &mut Self {
        self.cache_config = Some(config);
        self
    }

    /// Set conversation ID for xAI server routing.
    pub fn with_conv_id(mut self, conv_id: impl Into<String>) -> Self {
        let config = self.cache_config.unwrap_or_default();
        self.cache_config = Some(config.with_xai_conv_id(conv_id));
        self
    }

    /// Set prompt cache key for cache reuse.
    pub fn with_prompt_cache_key(mut self, key: impl Into<String>) -> Self {
        let config = self.cache_config.unwrap_or_default();
        self.cache_config = Some(config.with_prompt_cache_key(key));
        self
    }

    /// Get the current cache configuration.
    pub fn cache_config(&self) -> Option<&CacheConfig> {
        self.cache_config.as_ref()
    }
}

#[async_trait]
impl LanguageModel for XaiModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "xai"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(
        &self,
        prompt: Prompt,
        _options: GenerateOptions,
    ) -> AiResult<GenerateResult> {
        let text = match prompt {
            Prompt::Text(t) => t,
            _ => {
                return Err(AiError::UnsupportedCapability {
                    capability: "non-text prompts".to_string(),
                    provider: "xai".to_string(),
                })
            }
        };

        let request = ChatCompletionRequest {
            model: self.model_id.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: text,
            }],
            stream: Some(false),
            prompt_cache_key: self
                .cache_config
                .as_ref()
                .and_then(|c| c.prompt_cache_key.clone()),
        };

        let response = self
            .client
            .create_chat_completion(request, self.cache_config.as_ref())
            .await?;

        let text = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::ProviderError {
                provider: "xai".to_string(),
                status: None,
                message: "No content in response".to_string(),
            })?;

        let usage = response
            .usage
            .map(|u| Usage {
                prompt_tokens: Some(u.prompt_tokens as u64),
                completion_tokens: Some(u.completion_tokens as u64),
                total_tokens: Some(u.total_tokens as u64),
            })
            .unwrap_or_default();

        Ok(GenerateResult {
            text: Some(text),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage,
            metadata: Default::default(),
        })
    }

    async fn stream(
        &self,
        prompt: Prompt,
        _options: GenerateOptions,
    ) -> AiResult<rs_ai_core::AiStream> {
        let text = match prompt {
            Prompt::Text(t) => t,
            _ => {
                return Err(AiError::UnsupportedCapability {
                    capability: "non-text prompts".to_string(),
                    provider: "xai".to_string(),
                })
            }
        };

        let request = ChatCompletionRequest {
            model: self.model_id.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: text,
            }],
            stream: Some(true),
            prompt_cache_key: self
                .cache_config
                .as_ref()
                .and_then(|c| c.prompt_cache_key.clone()),
        };

        let response = self
            .client
            .create_chat_completion_stream(request, self.cache_config.as_ref())
            .await?;

        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, chunk| {
                futures::future::ready(match chunk {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        Some(Ok(buffer.clone()))
                    }
                    Err(e) => Some(Err::<_, reqwest::Error>(e)),
                })
            })
            .flat_map(|result| match result {
                Ok(buffer) => {
                    let events = parse_stream_buffer(&buffer);
                    futures::stream::iter(events)
                }
                Err(e) => {
                    let err = Err::<StreamEvent, AiError>(AiError::StreamError {
                        message: e.to_string(),
                    });
                    futures::stream::iter(vec![err])
                }
            });

        Ok(Box::pin(stream))
    }
}

fn parse_stream_buffer(buffer: &str) -> Vec<Result<StreamEvent, AiError>> {
    let mut events = Vec::new();

    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }

        if !line.starts_with("data: ") {
            continue;
        }

        let data = &line[6..];

        if data == "[DONE]" {
            events.push(Ok(StreamEvent::MessageEnd {
                finish_reason: FinishReason::Stop,
                usage: None,
            }));
            continue;
        }

        match serde_json::from_str::<serde_json::Value>(data) {
            Ok(value) => {
                if let Some(choices) = value.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        if let Some(delta) = choice.get("delta") {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    events.push(Ok(StreamEvent::TextDelta {
                                        delta: content.to_string(),
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Skip unparseable lines
            }
        }
    }

    events
}
