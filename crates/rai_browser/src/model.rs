use std::sync::Arc;

use async_trait::async_trait;
use rai_ai::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, ContentPart, FinishReason,
    GenerateOptions, GenerateResult, LanguageModel, Prompt, ResponseMetadata, SyntheticStreamer,
    Usage,
};

use crate::bridge::BrowserAiBridge;
use crate::capabilities::BrowserAiOptions;

/// A [`LanguageModel`] backed by the browser's built-in AI.
pub struct BrowserAiModel {
    bridge: Arc<dyn BrowserAiBridge>,
    capabilities: CapabilitySet,
}

impl BrowserAiModel {
    pub(crate) fn new(bridge: Arc<dyn BrowserAiBridge>) -> Self {
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
}

#[async_trait]
impl LanguageModel for BrowserAiModel {
    fn model_id(&self) -> &str {
        "browser-ai"
    }

    fn provider_id(&self) -> &str {
        "browser"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        let caps = self.bridge.detect().await;
        if !caps.available {
            return Err(AiError::PlatformUnavailable {
                platform: "Browser AI (no built-in AI detected)".into(),
            });
        }

        let browser_options = BrowserAiOptions {
            temperature: options.temperature,
            top_k: options.top_k,
            ..Default::default()
        };

        let text = Self::prompt_to_text(&prompt);
        let response = self
            .bridge
            .generate(&text, &browser_options)
            .await
            .map_err(|e| AiError::BridgeError {
                bridge: "browser".into(),
                message: e,
            })?;

        Ok(GenerateResult {
            text: Some(response),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata {
                provider: "browser".into(),
                model: "browser-ai".into(),
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
