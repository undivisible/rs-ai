use std::sync::Arc;

use async_trait::async_trait;

use rs_ai_traits::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, ContentPart, FinishReason,
    GenerateOptions, GenerateResult, LanguageModel, Message, Prompt, ResponseMetadata, Role,
    SyntheticStreamer, Usage,
};

use crate::bridge::GeminiNanoBridge;
use crate::types::NanoSessionConfig;

/// A language model backed by Gemini Nano running on-device via the Android
/// Prompt API.
///
/// Streaming is synthetic -- the full response is generated first and then
/// chunked into a stream, because Gemini Nano does not support native
/// streaming on Android.
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
            .with(Capability::LocalExecution)
            .with(Capability::SessionSupport)
            .with(Capability::PlatformNative);

        Self {
            bridge,
            capabilities,
        }
    }

    /// Build a [`NanoSessionConfig`] from the generic [`GenerateOptions`].
    fn build_config(options: &GenerateOptions) -> NanoSessionConfig {
        NanoSessionConfig {
            temperature: options.temperature,
            top_k: options.top_k,
            max_tokens: options.max_tokens,
        }
    }

    /// Check availability and return an error if the model is not ready.
    async fn ensure_available(&self) -> AiResult<()> {
        if !self.bridge.is_available().await {
            return Err(AiError::PlatformUnavailable {
                platform: "android/gemini_nano".into(),
            });
        }

        let state = self.bridge.download_state().await;
        match state {
            crate::types::ModelDownloadState::Downloaded => Ok(()),
            crate::types::ModelDownloadState::NotDownloaded => Err(AiError::ModelUnavailable {
                model: "gemini-nano (not downloaded)".into(),
            }),
            crate::types::ModelDownloadState::Downloading { progress_percent } => {
                Err(AiError::ModelUnavailable {
                    model: format!("gemini-nano (downloading: {progress_percent}%)"),
                })
            }
            crate::types::ModelDownloadState::Failed { reason } => Err(AiError::BridgeError {
                bridge: "gemini_nano".into(),
                message: format!("Model download failed: {reason}"),
            }),
        }
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
        let prompt_text = extract_prompt_text(prompt);
        let start = std::time::Instant::now();

        let text = self
            .bridge
            .generate(&prompt_text, &config)
            .await
            .map_err(|e| AiError::BridgeError {
                bridge: "gemini_nano".into(),
                message: e,
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(GenerateResult {
            text: Some(text),
            tool_calls: Vec::new(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata {
                provider: "gemini_nano".into(),
                model: "gemini-nano".into(),
                latency_ms: Some(latency_ms),
                ..Default::default()
            },
        })
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let result = self.generate(prompt, options).await?;
        let text = result.text.unwrap_or_default();
        Ok(SyntheticStreamer::stream(text, 20))
    }
}

/// Extract a single prompt string from a [`Prompt`] for the bridge.
fn extract_prompt_text(prompt: Prompt) -> String {
    match prompt {
        Prompt::Text(text) => text,
        Prompt::Messages(messages) => messages_to_text(&messages),
    }
}

/// Concatenate messages into a single string.
fn messages_to_text(messages: &[Message]) -> String {
    let mut parts = Vec::new();

    for msg in messages {
        let prefix = match msg.role {
            Role::System => "System: ",
            Role::User => "",
            Role::Assistant => "Assistant: ",
            Role::Tool => "Tool: ",
        };

        for part in &msg.content {
            if let ContentPart::Text { text } = part {
                if prefix.is_empty() {
                    parts.push(text.clone());
                } else {
                    parts.push(format!("{prefix}{text}"));
                }
            }
        }
    }

    parts.join("\n")
}
