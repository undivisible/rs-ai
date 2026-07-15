use async_trait::async_trait;
use futures::stream::StreamExt;
use rs_ai_core::CacheConfig;
use rs_ai_core::{
    AiError, AiResult, Capability, CapabilitySet, ContentPart, FinishReason, GenerateOptions,
    GenerateResult, LanguageModel, Prompt, Role, StreamEvent, Usage,
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

fn chat_messages(prompt: Prompt) -> AiResult<Vec<Message>> {
    prompt
        .into_messages()
        .into_iter()
        .map(|message| {
            let content = message.content.into_iter().try_fold(
                String::new(),
                |mut content, part| match part {
                    ContentPart::Text { text } => {
                        content.push_str(&text);
                        Ok(content)
                    }
                    _ => Err(AiError::UnsupportedCapability {
                        capability: "non-text message content".to_string(),
                        provider: "xai".to_string(),
                    }),
                },
            )?;
            Ok(Message {
                role: match message.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                }
                .to_string(),
                content,
            })
        })
        .collect()
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
        let request = ChatCompletionRequest {
            model: self.model_id.clone(),
            messages: chat_messages(prompt)?,
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
        let request = ChatCompletionRequest {
            model: self.model_id.clone(),
            messages: chat_messages(prompt)?,
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
            .scan(Vec::new(), |buffer, chunk| {
                futures::future::ready(match chunk {
                    Ok(bytes) => {
                        let events = parse_stream_chunk(buffer, &bytes);
                        Some(Ok(events))
                    }
                    Err(error) => Some(Err(AiError::StreamError {
                        message: error.to_string(),
                    })),
                })
            })
            .flat_map(|result| match result {
                Ok(events) => futures::stream::iter(events),
                Err(error) => futures::stream::iter(vec![Err(error)]),
            });

        Ok(Box::pin(stream))
    }
}

fn parse_stream_chunk(buffer: &mut Vec<u8>, bytes: &[u8]) -> Vec<Result<StreamEvent, AiError>> {
    buffer.extend_from_slice(bytes);
    let Some(end) = buffer.iter().rposition(|byte| *byte == b'\n') else {
        return Vec::new();
    };
    let completed: Vec<u8> = buffer.drain(..=end).collect();
    parse_stream_buffer(&String::from_utf8_lossy(&completed))
}

fn parse_stream_buffer(buffer: &str) -> Vec<Result<StreamEvent, AiError>> {
    let mut events = Vec::new();

    for line in buffer.lines() {
        if line.is_empty() {
            continue;
        }

        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim_start();

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

#[cfg(test)]
mod tests {
    use super::{chat_messages, parse_stream_chunk};
    use rs_ai_core::{Message, Prompt, StreamEvent};

    #[test]
    fn structured_prompt_preserves_chat_roles() {
        let messages = chat_messages(Prompt::Messages(vec![
            Message::system("Follow the context"),
            Message::assistant("I can help"),
            Message::user("Plan my day"),
        ]))
        .unwrap();

        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "Follow the context");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[2].role, "user");
    }

    #[test]
    fn split_sse_chunks_emit_each_delta_once() {
        let mut buffer = Vec::new();
        assert!(parse_stream_chunk(
            &mut buffer,
            br#"data: {"choices":[{"delta":{"content":"hel"#,
        )
        .is_empty());
        let events = parse_stream_chunk(
            &mut buffer,
            b"lo\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n",
        );
        let deltas = events
            .into_iter()
            .filter_map(|event| match event.unwrap() {
                StreamEvent::TextDelta { delta } => Some(delta),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(deltas, ["hello", "lo"]);
    }
}
