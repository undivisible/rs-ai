//! Example: Apple Foundation Models integration.
//!
//! This example demonstrates how the Apple Foundation Models provider would be
//! used once the `rs_ai_foundationmodels` crate is fully implemented.
//!
//! On a real Apple device the bridge would call into the Foundation Models
//! framework via Swift/Objective-C interop. Here we show the intended usage
//! pattern with a mock bridge that mirrors the Gemini Nano example.

use async_trait::async_trait;
use rs_ai_ai::capability::{Capability, CapabilitySet};
use rs_ai_ai::error::{AiError, AiResult};
use rs_ai_ai::model::{GenerateOptions, LanguageModel};
use rs_ai_ai::prompt::Prompt;
use rs_ai_ai::stream::{AiStream, SyntheticStreamer};
use rs_ai_ai::structured::GenerateResult;
use rs_ai_ai::types::{FinishReason, ResponseMetadata};
use rs_ai_ai::usage::Usage;
use rs_ai_ai::*;

/// Trait representing the bridge to Apple Foundation Models on-device runtime.
#[async_trait]
trait FoundationModelsBridge: Send + Sync {
    async fn is_available(&self) -> bool;
    async fn generate(&self, prompt: &str) -> Result<String, String>;
}

/// Mock bridge for demonstration purposes.
struct MockAppleBridge;

#[async_trait]
impl FoundationModelsBridge for MockAppleBridge {
    async fn is_available(&self) -> bool {
        true
    }

    async fn generate(&self, prompt: &str) -> Result<String, String> {
        Ok(format!(
            "[Apple Foundation Models mock response to: {prompt}]"
        ))
    }
}

/// A language model backed by Apple Foundation Models.
struct FoundationModel {
    bridge: Box<dyn FoundationModelsBridge>,
    capabilities: CapabilitySet,
}

impl FoundationModel {
    fn new(bridge: impl FoundationModelsBridge + 'static) -> Self {
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
impl LanguageModel for FoundationModel {
    fn model_id(&self) -> &str {
        "apple-foundation-model"
    }

    fn provider_id(&self) -> &str {
        "apple"
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
                platform: "apple/foundation_models".into(),
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
                bridge: "apple_foundation_models".into(),
                message: e,
            })?;

        Ok(GenerateResult {
            text: Some(text),
            tool_calls: Vec::new(),
            finish_reason: FinishReason::Stop,
            usage: Usage::default(),
            metadata: ResponseMetadata {
                provider: "apple".into(),
                model: "apple-foundation-model".into(),
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
    let model = FoundationModel::new(MockAppleBridge);

    // Check availability
    println!(
        "Apple Foundation Model available: {}",
        model.bridge.is_available().await
    );

    // Generate text
    let result = generate_text(&model, "Explain Swift concurrency in one paragraph").await?;
    println!("Response: {result}");

    // Use via the LanguageModel trait with custom options
    let options = GenerateOptions::default().with_temperature(0.7);
    let result = model
        .generate(
            Prompt::from("What is the latest version of macOS?"),
            options,
        )
        .await?;
    if let Some(text) = &result.text {
        println!("With options: {text}");
    }

    Ok(())
}
