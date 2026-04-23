//! Example: Windows Phi Silica integration.
//!
//! This example demonstrates how the Phi Silica provider would be used once the
//! `rai_phi_silica` crate is fully implemented.
//!
//! On a real Windows device the bridge would call into the Windows Copilot
//! Runtime via the Windows App SDK. Here we show the intended usage pattern
//! with a mock bridge that mirrors the Gemini Nano example.

use async_trait::async_trait;
use rai_ai::capability::{Capability, CapabilitySet};
use rai_ai::error::{AiError, AiResult};
use rai_ai::model::{GenerateOptions, LanguageModel};
use rai_ai::prompt::Prompt;
use rai_ai::stream::{AiStream, SyntheticStreamer};
use rai_ai::structured::GenerateResult;
use rai_ai::types::{FinishReason, ResponseMetadata};
use rai_ai::usage::Usage;
use rai_ai::*;

/// Trait representing the bridge to Phi Silica on Windows.
#[async_trait]
trait PhiSilicaBridge: Send + Sync {
    async fn is_available(&self) -> bool;
    async fn generate(&self, prompt: &str) -> Result<String, String>;
}

/// Mock bridge for demonstration purposes.
struct MockPhiSilicaBridge;

#[async_trait]
impl PhiSilicaBridge for MockPhiSilicaBridge {
    async fn is_available(&self) -> bool {
        true
    }

    async fn generate(&self, prompt: &str) -> Result<String, String> {
        Ok(format!("[Phi Silica mock response to: {prompt}]"))
    }
}

/// A language model backed by Phi Silica on Windows.
struct PhiSilicaModel {
    bridge: Box<dyn PhiSilicaBridge>,
    capabilities: CapabilitySet,
}

impl PhiSilicaModel {
    fn new(bridge: impl PhiSilicaBridge + 'static) -> Self {
        let capabilities = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::LocalExecution)
            .with(Capability::PlatformNative);
        Self {
            bridge: Box::new(bridge),
            capabilities,
        }
    }
}

#[async_trait]
impl LanguageModel for PhiSilicaModel {
    fn model_id(&self) -> &str {
        "phi-silica"
    }

    fn provider_id(&self) -> &str {
        "windows"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(
        &self,
        prompt: Prompt,
        _options: GenerateOptions,
    ) -> AiResult<GenerateResult> {
        if !self.bridge.is_available().await {
            return Err(AiError::PlatformUnavailable {
                platform: "windows/phi_silica".into(),
            });
        }

        let prompt_text = match prompt {
            Prompt::Text(t) => t,
            Prompt::Messages(msgs) => msgs
                .into_iter()
                .filter_map(|m| {
                    m.content.into_iter().find_map(|part| {
                        if let ContentPart::Text { text } = part {
                            Some(text)
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let text = self
            .bridge
            .generate(&prompt_text)
            .await
            .map_err(|e| AiError::BridgeError {
                bridge: "phi_silica".into(),
                message: e,
            })?;

        Ok(GenerateResult {
            text: Some(text),
            tool_calls: Vec::new(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata {
                provider: "windows".into(),
                model: "phi-silica".into(),
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = PhiSilicaModel::new(MockPhiSilicaBridge);

    // Check availability
    println!(
        "Phi Silica available: {}",
        model.bridge.is_available().await
    );

    // Generate text
    let result = generate_text(&model, "What is the Windows Copilot Runtime?").await?;
    println!("Response: {result}");

    // Use via the LanguageModel trait with custom options
    let options = GenerateOptions::default().with_max_tokens(256);
    let result = model
        .generate(
            Prompt::from("Describe the Phi model family in one paragraph"),
            options,
        )
        .await?;
    if let Some(text) = &result.text {
        println!("With options: {text}");
    }

    Ok(())
}
