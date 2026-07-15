use async_trait::async_trait;
use futures::stream::StreamExt;
use rs_ai_core::{
    AiError, AiResult, Capability, CapabilitySet, FinishReason, GenerateOptions, GenerateResult,
    LanguageModel, Prompt, StreamEvent, Usage,
};

use super::client::{ChatCompletionRequest, CloudflareClient, Message};

/// Cloudflare Workers AI model.
#[derive(Clone)]
pub struct CloudflareModel {
    model_id: String,
    client: std::sync::Arc<CloudflareClient>,
    capabilities: CapabilitySet,
}

impl CloudflareModel {
    /// Create a new Cloudflare model instance.
    pub fn new(model_id: String, client: std::sync::Arc<CloudflareClient>) -> Self {
        let capabilities = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::Streaming);

        Self {
            model_id,
            client,
            capabilities,
        }
    }
}

#[async_trait]
impl LanguageModel for CloudflareModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "cloudflare"
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
                    provider: "cloudflare".to_string(),
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
        };

        let response = self.client.create_chat_completion(request).await?;

        let text = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::ProviderError {
                provider: "cloudflare".to_string(),
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
        steps: Vec::new(),
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
                    provider: "cloudflare".to_string(),
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
        };

        let response = self.client.create_chat_completion_stream(request).await?;

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

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
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
    }

    events
}
