use std::sync::Arc;

use async_trait::async_trait;

use rs_ai_core::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, ContentPart, FinishReason,
    GenerateOptions, GenerateResult, ImageData, LanguageModel, Message, Prompt, ResponseMetadata,
    SyntheticStreamer, Usage,
};

use super::bridge::GeminiNanoBridge;
use super::types::{NanoContentPart, NanoGenerationConfig};

/// A language model backed by Gemini Nano running on-device via the
/// ML Kit GenAI Prompt API.
///
/// Streaming is synthetic — the full response is generated first and then
/// chunked into a stream, because the GenAI Prompt API does not support
/// native streaming.
pub struct GeminiNanoModel {
    bridge: Arc<dyn GeminiNanoBridge>,
    capabilities: CapabilitySet,
}

impl GeminiNanoModel {
    /// Create a new `GeminiNanoModel` wrapping the given bridge.
    pub fn new(bridge: Arc<dyn GeminiNanoBridge>) -> Self {
        let capabilities = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::ImageInput)
            .with(Capability::LocalExecution)
            .with(Capability::PlatformNative);

        Self {
            bridge,
            capabilities,
        }
    }

    /// Build a [`NanoGenerationConfig`] from the generic [`GenerateOptions`].
    fn build_config(options: &GenerateOptions) -> NanoGenerationConfig {
        NanoGenerationConfig {
            temperature: options.temperature.map(|t| t as f32),
            max_output_tokens: options.max_tokens,
            ..Default::default()
        }
    }

    /// Check availability and return an error if the model is not ready.
    async fn ensure_available(&self) -> AiResult<()> {
        if !self.bridge.is_available().await {
            return Err(AiError::PlatformUnavailable {
                platform: "android/gemini_nano".into(),
            });
        }
        Ok(())
    }
}

#[async_trait]
impl LanguageModel for GeminiNanoModel {
    fn model_id(&self) -> &str {
        "gemini-nano"
    }

    fn provider_id(&self) -> &str {
        "gemini_nano"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        self.ensure_available().await?;

        let config = Self::build_config(&options);
        let parts = extract_content_parts(prompt);
        let start = std::time::Instant::now();

        let result = self
            .bridge
            .generate_content(parts, &config)
            .await
            .map_err(|e| AiError::BridgeError {
                bridge: "gemini_nano".into(),
                message: e,
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(GenerateResult {
            text: Some(result.text),
            tool_calls: Vec::new(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata {
                provider: "gemini_nano".into(),
                model: "gemini-nano".into(),
                latency_ms: Some(latency_ms),
                ..Default::default()
            },
            steps: Vec::new(),
            reasoning: None,
        })
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let result = self.generate(prompt, options).await?;
        let text = result.text.unwrap_or_default();
        Ok(SyntheticStreamer::stream(text, 20))
    }
}

/// Extract content parts from a [`Prompt`] for the bridge.
///
/// Handles text and base64-encoded images. URL images are skipped with a
/// warning log (would require downloading before passing to ML Kit).
fn extract_content_parts(prompt: Prompt) -> Vec<NanoContentPart> {
    match prompt {
        Prompt::Text(text) => vec![NanoContentPart::Text(text)],
        Prompt::Messages(messages) => messages_to_parts(&messages),
    }
}

/// Convert [`Message`]s into [`NanoContentPart`]s.
fn messages_to_parts(messages: &[Message]) -> Vec<NanoContentPart> {
    let mut parts = Vec::new();

    for msg in messages {
        for part in &msg.content {
            match part {
                ContentPart::Text { text } => {
                    if !parts.is_empty() {
                        // Separate messages with newline
                        parts.push(NanoContentPart::Text("\n".to_string()));
                    }
                    parts.push(NanoContentPart::Text(text.clone()));
                }
                ContentPart::Image { data } => match data {
                    ImageData::Base64 { media_type, data } => {
                        parts.push(NanoContentPart::Image {
                            base64: data.clone(),
                            mime_type: media_type.clone(),
                        });
                    }
                    ImageData::Url { url, .. } => {
                        // ponytail: skip URL images — would need to download
                        // before passing bitmap to ML Kit. Add when URL fetch
                        // is plumbed through the bridge.
                        tracing::warn!(
                            "Skipping URL image in Gemini Nano prompt: {url}. \
                             Only base64-encoded images are supported."
                        );
                    }
                },
                _ => {
                    // Skip tool calls, tool results, files
                }
            }
        }
    }

    parts
}
