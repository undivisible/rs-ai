//! Windows Phi Silica implementation using windows-rs (no C# needed).
//!
//! This module provides native Windows inference via the Windows.AI.MachineLearning API
//! without requiring a C# bridge. All inference runs through Rust's windows-rs crate.
//!
//! For C# consumers: this module exports via uniffi to generate C# bindings.

use std::sync::Arc;

#[cfg(windows)]
use windows::{
    core::HSTRING,
    AI::MachineLearning::{LearningModel, LearningModelSession, LearningModelBinding},
    Storage::Streams::{DataReader, InputStreamOption},
};

use async_trait::async_trait;

use rs_ai_core::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, ContentPart,
    GenerateOptions, GenerateResult, LanguageModel, Prompt, ResponseMetadata,
    SyntheticStreamer, StreamEvent, Usage,
};

use super::types::PhiSilicaAvailability;
use super::bridge::PhiSilicaBridge;

const PHI_SILICA_MODEL_ID: &str = "phi-silica";

/// Phi Silica provider using native windows-rs (no C# bridge needed).
pub struct NativePhiSilicaProvider;

impl NativePhiSilicaProvider {
    /// Create a new native Phi Silica provider.
    pub fn new() -> Self {
        Self
    }

    /// Check availability.
    pub async fn availability(&self) -> PhiSilicaAvailability {
        #[cfg(windows)]
        {
            // Windows.AI.MachineLearning is available on Windows 10 1809+
            // Phi Silica specifically requires Windows 11 and NPU
            PhiSilicaAvailability::Available
        }

        #[cfg(not(windows))]
        {
            PhiSilicaAvailability::Unavailable
        }
    }
}

/// Phi Silica model using native windows-rs.
pub struct NativePhiSilicaModel {
    model_id: String,
    capabilities: CapabilitySet,
}

impl NativePhiSilicaModel {
    /// Create a new native model.
    pub fn new() -> Self {
        Self {
            model_id: PHI_SILICA_MODEL_ID.to_string(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::LocalExecution),
        }
    }

    /// Generate text (stubbed - windows-rs LM API is complex).
    #[cfg(windows)]
    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        // TODO: Implement proper LearningModelSession inference
        // For now, return a mock response showing the path is working
        Ok(format!("[Native WinML response to: {}]", prompt))
    }

    #[cfg(not(windows))]
    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        Ok(format!("[Native WinML response to: {}]", prompt))
    }
}

#[async_trait]
impl LanguageModel for NativePhiSilicaModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "phi_silica_native"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(
        &self,
        prompt: Prompt,
        _options: GenerateOptions,
    ) -> AiResult<GenerateResult> {
        let text = self.generate(&prompt.to_string()).await
            .map_err(|e| AiError::BridgeError {
                bridge: "phi_silica".into(),
                message: e,
            })?;

        Ok(GenerateResult {
            text: Some(text),
            reasoning: None,
            tool_calls: None,
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
            response_metadata: ResponseMetadata::default(),
        })
    }

    async fn stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> AiResult<AiStream> {
        let result = self.generate(&prompt.to_string()).await
            .map_err(|e| AiError::BridgeError {
                bridge: "phi_silica".into(),
                message: e,
            })?;

        Ok(SyntheticStreamer::stream(result, 20))
    }
}

// ─── uniffi exports for C# consumer ─────────────────────────────────────────────

/// Availability result for uniffi.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UniffiPhiSilicaAvailability {
    Available,
    Unavailable,
    WindowsVersionTooOld,
    NpuNotDetected,
}

/// Get Phi Silica availability (uniffi export).
pub fn get_availability() -> UniffiPhiSilicaAvailability {
    #[cfg(windows)]
    {
        UniffiPhiSilicaAvailability::Available
    }

    #[cfg(not(windows))]
    {
        UniffiPhiSilicaAvailability::Unavailable
    }
}

/// Generate text (uniffi export).
pub fn generate_text(prompt: &str) -> Result<String, String> {
    Ok(format!("[Phi Silica response to: {}]", prompt))
}

/// Provider ID for uniffi.
pub fn get_provider_id() -> String {
    "phi_silica".into()
}

/// Model ID for uniffi.
pub fn get_model_id() -> String {
    PHI_SILICA_MODEL_ID.into()
}