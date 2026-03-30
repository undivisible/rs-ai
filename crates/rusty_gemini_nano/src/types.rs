/// The download state of the Gemini Nano model on the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelDownloadState {
    /// The model has not been downloaded to the device.
    NotDownloaded,
    /// The model is currently being downloaded.
    Downloading {
        /// Download progress as a percentage (0-100).
        progress_percent: u8,
    },
    /// The model has been downloaded and is ready for use.
    Downloaded,
    /// The model download failed.
    Failed {
        /// A human-readable reason for the failure.
        reason: String,
    },
}

/// Capabilities exposed by Gemini Nano on the current device.
#[derive(Debug, Clone)]
pub struct NanoCapabilities {
    /// Whether text generation is supported.
    pub text_generation: bool,
    /// Whether summarization is supported.
    pub summarization: bool,
    /// Whether rewriting is supported.
    pub rewriting: bool,
}

/// Configuration for a Gemini Nano session.
#[derive(Debug, Clone)]
pub struct NanoSessionConfig {
    /// Sampling temperature (0.0 - 1.0).
    pub temperature: Option<f64>,
    /// Top-k sampling parameter.
    pub top_k: Option<u32>,
    /// Maximum number of tokens to generate.
    pub max_tokens: Option<u32>,
}

impl Default for NanoSessionConfig {
    fn default() -> Self {
        Self {
            temperature: None,
            top_k: None,
            max_tokens: None,
        }
    }
}
