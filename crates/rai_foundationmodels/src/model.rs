use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;

use rai_ai::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, ContentPart, FinishReason,
    GenerateOptions, GenerateResult, LanguageModel, Prompt, ResponseMetadata, StreamEvent,
    SyntheticStreamer, Usage,
};

use crate::bridge::FoundationModelBridge;
use crate::types::{AppleModelAvailability, FoundationModelConfig};

/// A [`LanguageModel`] backed by Apple Foundation Models running on-device.
///
/// When the bridge supports streaming, real chunks are emitted as stream
/// events. Otherwise, [`SyntheticStreamer`] is used as a fallback.
pub struct FoundationModel {
    bridge: Arc<dyn FoundationModelBridge>,
    capabilities: CapabilitySet,
}

impl FoundationModel {
    pub(crate) fn new(bridge: Arc<dyn FoundationModelBridge>) -> Self {
        let capabilities = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::LocalExecution)
            .with(Capability::PlatformNative);
        Self {
            bridge,
            capabilities,
        }
    }

    fn build_config(options: &GenerateOptions) -> FoundationModelConfig {
        FoundationModelConfig {
            temperature: options.temperature,
            max_tokens: options.max_tokens,
        }
    }

    fn prompt_to_text(prompt: &Prompt) -> String {
        match prompt {
            Prompt::Text(t) => t.clone(),
            Prompt::Messages(msgs) => msgs
                .iter()
                .flat_map(|m| {
                    m.content.iter().filter_map(|c| match c {
                        ContentPart::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    async fn ensure_available(&self) -> AiResult<()> {
        match self.bridge.availability().await {
            AppleModelAvailability::Available => Ok(()),
            AppleModelAvailability::Unavailable { reason } => Err(AiError::PlatformUnavailable {
                platform: format!("apple/foundation_models: {reason}"),
            }),
            AppleModelAvailability::NeedsDownload => Err(AiError::ModelUnavailable {
                model: "apple-foundation-model (needs download)".into(),
            }),
        }
    }
}

#[async_trait]
impl LanguageModel for FoundationModel {
    fn model_id(&self) -> &str {
        "apple-foundation-model"
    }

    fn provider_id(&self) -> &str {
        "foundationmodels"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        self.ensure_available().await?;

        let config = Self::build_config(&options);
        let prompt_text = Self::prompt_to_text(&prompt);
        let start = std::time::Instant::now();

        let response = self
            .bridge
            .generate(&prompt_text, &config)
            .await
            .map_err(|e| AiError::BridgeError {
                bridge: "foundationmodels".into(),
                message: e,
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(GenerateResult {
            text: Some(response),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata {
                provider: "foundationmodels".into(),
                model: "apple-foundation-model".into(),
                latency_ms: Some(latency_ms),
                ..Default::default()
            },
        })
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        self.ensure_available().await?;

        let config = Self::build_config(&options);
        let prompt_text = Self::prompt_to_text(&prompt);

        // Try the bridge's streaming method first (AsyncSequence-backed).
        match self.bridge.stream(&prompt_text, &config).await {
            Ok(chunks) if !chunks.is_empty() => {
                let mut events: Vec<Result<StreamEvent, AiError>> = Vec::new();
                events.push(Ok(StreamEvent::MessageStart {
                    message_id: uuid::Uuid::new_v4().to_string(),
                }));
                for chunk in chunks {
                    events.push(Ok(StreamEvent::TextDelta { delta: chunk }));
                }
                events.push(Ok(StreamEvent::MessageEnd {
                    finish_reason: FinishReason::Stop,
                    usage: None,
                }));
                Ok(Box::pin(stream::iter(events)))
            }
            _ => {
                // Fallback: generate fully and use synthetic streaming.
                let result = self.generate(Prompt::Text(prompt_text), options).await?;
                let text = result.text.unwrap_or_default();
                Ok(SyntheticStreamer::stream(text, 20))
            }
        }
    }
}
