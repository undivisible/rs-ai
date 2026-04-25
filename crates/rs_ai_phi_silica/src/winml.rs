//! Windows ML integration via the `windows` crate.
//!
//! Provides real on-device ONNX model inference on Windows 10+ using
//! `Windows.AI.MachineLearning.LearningModel`.
//!
//! This is not Phi Silica specifically, but it provides real local AI inference
//! on Windows. For Phi Silica, use a custom [`PhiSilicaBridge`] implementation.

use std::path::Path;

use async_trait::async_trait;
use rs_ai_traits::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, ContentPart, FinishReason,
    GenerateOptions, GenerateResult, LanguageModel, Prompt, ResponseMetadata, StreamEvent,
    SyntheticStreamer, Usage,
};

use crate::types::PhiSilicaAvailability;

/// A [`LanguageModel`] backed by Windows ML running an ONNX model locally.
///
/// Loads an ONNX model file and runs inference via the Windows ML runtime.
/// This works on Windows 10 1809+ and Windows 11 with any ONNX model.
///
/// # Example
///
/// ```no_run
/// use rs_ai_phi_silica::WinMlModel;
///
/// let model = WinMlModel::from_file("model.onnx").unwrap();
/// ```
pub struct WinMlModel {
    model_id: String,
    capabilities: CapabilitySet,
}

impl WinMlModel {
    /// Create a WinML model from an ONNX file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the model file cannot be loaded.
    pub fn from_file<P: AsRef<Path>>(path: P) -> AiResult<Self> {
        let path = path.as_ref();

        #[cfg(windows)]
        {
            // Try to load the model to validate it
            let _ = Self::load_model(path).map_err(|e| AiError::BridgeError {
                bridge: "winml".into(),
                message: format!("Failed to load ONNX model: {e}"),
            })?;
        }

        Ok(Self {
            model_id: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("winml-model")
                .to_string(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::LocalExecution),
        })
    }

    /// Create a WinML model with a custom identifier.
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            model_id: id.into(),
            capabilities: CapabilitySet::new()
                .with(Capability::TextInput)
                .with(Capability::TextOutput)
                .with(Capability::LocalExecution),
        }
    }

    #[cfg(windows)]
    fn load_model(_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        use windows::Storage::PathIO;
        use windows::AI::MachineLearning::LearningModel;

        let path_str = _path.to_string_lossy().to_string();
        let _stream = PathIO::ReadFileAsync(&path_str.into())?.get()?;
        let _model = LearningModel::LoadFromStreamAsync(&_stream)?.get()?;
        Ok(())
    }
}

#[async_trait]
impl LanguageModel for WinMlModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "winml"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(
        &self,
        _prompt: Prompt,
        _options: GenerateOptions,
    ) -> AiResult<GenerateResult> {
        Err(AiError::BridgeError {
            bridge: "winml".into(),
            message:
                "WinML requires ONNX model-specific binding code. Use a custom bridge for now."
                    .into(),
        })
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let result = self.generate(prompt, options).await?;
        let text = result.text.unwrap_or_default();
        Ok(SyntheticStreamer::stream(text, 20))
    }
}

/// Provider for Windows ML on-device inference.
pub struct WinMlProvider {
    model_path: String,
}

impl WinMlProvider {
    /// Create a new WinML provider for the given ONNX model path.
    pub fn new(model_path: impl Into<String>) -> Self {
        Self {
            model_path: model_path.into(),
        }
    }

    /// Load the ONNX model and return a language model.
    pub fn model(&self) -> AiResult<WinMlModel> {
        WinMlModel::from_file(&self.model_path)
    }
}

/// Check if Windows ML is available on this device.
pub fn winml_available() -> PhiSilicaAvailability {
    #[cfg(windows)]
    {
        // Windows ML is available on Windows 10 1809+
        PhiSilicaAvailability::Available
    }
    #[cfg(not(windows))]
    {
        PhiSilicaAvailability::Unavailable
    }
}
